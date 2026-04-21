from __future__ import annotations

import json
import subprocess
import threading
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any


WATCHDOG_MARGIN_SECONDS = 30


ACTION_PROTOCOL = """\
You are controlling a single Bash tool in a benchmark harness.
Respond with exactly one JSON object and nothing else.

Valid response shapes:
{"type":"bash","command":"<single shell command>"}
{"type":"final","output":<string or JSON value>}

Rules:
- Emit exactly one Bash command per turn.
- Do not use command separators like ;, &&, ||, or embedded newlines.
- Do not wrap the JSON in markdown fences.
- When the task requires JSON output, you may set output to an object or array.
- When the task requires text output, set output to the exact final text.
- For file-only tasks, return {"type":"final","output":"DONE"} after the file is correct.
"""


@dataclass
class LoopTrace:
    raw_output: str
    text_outputs: list[str]
    tool_outputs: list[str]
    bytes_output: int
    bytes_observation: int
    tool_calls: int
    turns: int
    invalid_responses: int = 0


@dataclass
class LoopAction:
    action_type: str
    command: str | None = None
    output: Any = None


def normalize_api_base(api_base: str) -> str:
    base = api_base.rstrip("/")
    if not base.endswith("/v1"):
        base = f"{base}/v1"
    return base


def resolve_oai_model(api_base: str, api_key: str, requested_model: str | None) -> str:
    if requested_model:
        return requested_model

    data = _request_json(normalize_api_base(api_base), api_key, "/models", None, 10)
    models = data.get("data", [])
    if not isinstance(models, list) or not models:
        raise RuntimeError("OAI API returned no models from /models")

    first = models[0]
    if not isinstance(first, dict) or not isinstance(first.get("id"), str):
        raise RuntimeError("OAI API returned an invalid model list payload")

    return first["id"]


def parse_action_text(raw_text: str) -> LoopAction:
    candidate = _extract_last_json_object(raw_text)
    parsed = json.loads(candidate)
    if not isinstance(parsed, dict):
        raise ValueError("response must be a JSON object")

    action_type = parsed.get("type")
    if action_type == "bash":
        command = parsed.get("command")
        if not isinstance(command, str) or not command.strip():
            raise ValueError("bash response must include a non-empty string command")
        if any(token in command for token in ("\n", ";", "&&", "||")):
            raise ValueError("bash command must be a single shell command without separators")
        return LoopAction(action_type="bash", command=command.strip())

    if action_type == "final":
        return LoopAction(action_type="final", output=parsed.get("output", ""))

    raise ValueError("response type must be 'bash' or 'final'")


def format_final_output(output: Any) -> str:
    if isinstance(output, str):
        return output
    return json.dumps(output, ensure_ascii=False)


def run_oai_loop(
    *,
    api_base: str,
    api_key: str,
    model: str,
    prompt: str,
    workdir: Path,
    env: dict[str, str],
    max_turns: int = 30,
    request_timeout_seconds: int = 60,
    tool_timeout_seconds: int = 30,
) -> LoopTrace:
    base = normalize_api_base(api_base)
    messages = [
        {"role": "system", "content": ACTION_PROTOCOL},
        {"role": "user", "content": prompt},
    ]

    transcript: list[dict[str, Any]] = []
    text_outputs: list[str] = []
    tool_outputs: list[str] = []
    bytes_output = 0
    bytes_observation = 0
    tool_calls = 0
    invalid_responses = 0

    for turn in range(1, max_turns + 1):
        response = _request_json(
            base,
            api_key,
            "/chat/completions",
            {
                "model": model,
                "messages": messages,
                "temperature": 0,
                "max_tokens": 1024,
                "response_format": {"type": "json_object"},
            },
            request_timeout_seconds,
        )
        assistant_text = _extract_assistant_text(response)
        bytes_output += len(assistant_text.encode())
        transcript.append({"turn": turn, "assistant": assistant_text})

        try:
            action = parse_action_text(assistant_text)
        except (json.JSONDecodeError, ValueError) as exc:
            invalid_responses += 1
            correction = (
                "Your last response was invalid. "
                "Reply again with exactly one JSON object matching the required schema. "
                f"Error: {exc}"
            )
            messages.append({"role": "assistant", "content": assistant_text})
            messages.append({"role": "user", "content": correction})
            transcript.append({"turn": turn, "user": correction})
            continue

        messages.append({"role": "assistant", "content": assistant_text})

        if action.action_type == "final":
            final_output = format_final_output(action.output)
            text_outputs.append(final_output)
            transcript.append({"turn": turn, "final": final_output})
            return LoopTrace(
                raw_output=json.dumps(transcript, indent=2, ensure_ascii=False),
                text_outputs=text_outputs,
                tool_outputs=tool_outputs,
                bytes_output=bytes_output,
                bytes_observation=bytes_observation,
                tool_calls=tool_calls,
                turns=turn,
                invalid_responses=invalid_responses,
            )

        tool_calls += 1
        observation = _run_bash_command(
            action.command or "",
            workdir=workdir,
            env=env,
            timeout_seconds=tool_timeout_seconds,
        )
        tool_outputs.append(observation)
        bytes_observation += len(observation.encode())
        transcript.append({"turn": turn, "tool_result": observation})
        messages.append({"role": "user", "content": f"Tool result:\n{observation}"})

    timeout_note = "MAX_TURNS_EXCEEDED"
    text_outputs.append(timeout_note)
    transcript.append({"turn": max_turns, "final": timeout_note})
    return LoopTrace(
        raw_output=json.dumps(transcript, indent=2, ensure_ascii=False),
        text_outputs=text_outputs,
        tool_outputs=tool_outputs,
        bytes_output=bytes_output,
        bytes_observation=bytes_observation,
        tool_calls=tool_calls,
        turns=max_turns,
        invalid_responses=invalid_responses,
    )


def _request_json(
    api_base: str,
    api_key: str,
    path: str,
    payload: dict[str, Any] | None,
    timeout_seconds: int,
) -> dict[str, Any]:
    # urllib's socket timeout doesn't reliably fire on streaming chat completions
    # when keepalives reset the read timer (observed on Hermes-4-70B-4bit via OMLX:
    # the harness sat on a single POST for 10+ minutes with timeout=300). Enforce
    # a hard wall-time bound in a worker thread so a stuck request surfaces as a
    # bounded TimeoutError the harness can record as runner_error instead of a
    # silent hang that forces the whole iteration to be re-run.
    deadline = max(timeout_seconds + WATCHDOG_MARGIN_SECONDS, 1)
    result_holder: list[dict[str, Any]] = []
    error_holder: list[BaseException] = []

    def _worker() -> None:
        try:
            result_holder.append(
                _do_request_json(api_base, api_key, path, payload, timeout_seconds)
            )
        except BaseException as exc:  # noqa: BLE001 — must capture everything the worker raises
            error_holder.append(exc)

    thread = threading.Thread(target=_worker, daemon=True, name="oai-req")
    thread.start()
    thread.join(timeout=deadline)

    if thread.is_alive():
        raise TimeoutError(
            f"OAI request to {path} exceeded wall-time deadline of {deadline}s "
            f"(socket timeout={timeout_seconds}s did not fire)"
        )
    if error_holder:
        raise error_holder[0]
    if not result_holder:
        raise RuntimeError(f"OAI request to {path} returned no result")
    return result_holder[0]


def _do_request_json(
    api_base: str,
    api_key: str,
    path: str,
    payload: dict[str, Any] | None,
    timeout_seconds: int,
) -> dict[str, Any]:
    body = None if payload is None else json.dumps(payload).encode()
    request = urllib.request.Request(
        f"{api_base}{path}",
        data=body,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="GET" if payload is None else "POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            return json.load(response)
    except urllib.error.HTTPError as exc:
        error_body = exc.read().decode(errors="replace")
        raise RuntimeError(f"OAI request failed with HTTP {exc.code}: {error_body}") from exc


def _extract_assistant_text(response: dict[str, Any]) -> str:
    choices = response.get("choices")
    if not isinstance(choices, list) or not choices:
        raise RuntimeError(f"chat completion response missing choices: {response}")

    message = choices[0].get("message")
    if not isinstance(message, dict):
        raise RuntimeError(f"chat completion response missing message: {response}")

    content = message.get("content", "")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts: list[str] = []
        for item in content:
            if isinstance(item, dict) and isinstance(item.get("text"), str):
                parts.append(item["text"])
        return "\n".join(parts)
    return str(content)


def _extract_last_json_object(text: str) -> str:
    clean = text.strip()
    if clean.startswith("```"):
        clean = clean.strip("`")
        clean = clean.replace("json\n", "", 1).strip()

    try:
        parsed = json.loads(clean)
        if isinstance(parsed, dict):
            return clean
    except json.JSONDecodeError:
        pass

    candidates: list[tuple[int, str]] = []
    depth = 0
    start = -1
    in_string = False
    escape = False
    for index, char in enumerate(clean):
        if escape:
            escape = False
            continue
        if char == "\\" and in_string:
            escape = True
            continue
        if char == "\"":
            in_string = not in_string
            continue
        if in_string:
            continue
        if char == "{":
            if depth == 0:
                start = index
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0 and start >= 0:
                candidate = clean[start:index + 1]
                try:
                    parsed = json.loads(candidate)
                    if isinstance(parsed, dict):
                        candidates.append((start, candidate))
                except json.JSONDecodeError:
                    pass
                start = -1

    if candidates:
        return candidates[-1][1]
    raise ValueError("response did not contain a JSON object")


def _run_bash_command(
    command: str,
    *,
    workdir: Path,
    env: dict[str, str],
    timeout_seconds: int,
) -> str:
    try:
        result = subprocess.run(
            ["/bin/bash", "-lc", command],
            cwd=workdir,
            env=env,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
        stdout = result.stdout or ""
        stderr = result.stderr or ""
        exit_code = result.returncode
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout or ""
        stderr = exc.stderr or ""
        exit_code = 124

    return (
        f"COMMAND: {command}\n"
        f"EXIT_CODE: {exit_code}\n"
        f"STDOUT:\n{stdout if stdout else '(empty)'}\n"
        f"STDERR:\n{stderr if stderr else '(empty)'}"
    )
