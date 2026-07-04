import { appendFile, mkdir, readFile, readFileSync, writeFile } from "node:fs";
import { dirname, isAbsolute, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const AUDIT_SCHEMA = "pi-audit.v1";
const SPEC_PATH = process.env.BENCH_MULTIFILE_DRIFT_SPECS?.trim() || fileURLToPath(new URL("../drift_specs.json", import.meta.url));
const SPECS = JSON.parse(readFileSync(SPEC_PATH, "utf8"));
const SECRET_KEY_RE = /(?:key|token|secret|password|credential|auth|api[_-]?key)$/i;

let observedRead = false;
let driftFired = false;
let targetMutationSeen = false;

export default function multifileDriftAuditExtension(pi) {
	pi.on("session_start", async (_event, ctx) => {
		observedRead = false;
		driftFired = false;
		targetMutationSeen = false;

		const model = ctx.model;
		if (model) {
			await mirror(ctx, {
				event: "model_change",
				details: {
					provider: model.provider,
					modelId: model.id,
					name: model.name,
					reasoning: model.reasoning,
				},
			});
		}
		await mirror(ctx, {
			event: "thinking_level_change",
			details: { thinkingLevel: pi.getThinkingLevel() },
		});
		await mirror(ctx, {
			event: "multifile_drift_config",
			details: driftConfig(ctx),
		});
	});

	pi.on("tool_call", async (event, ctx) => {
		await mirror(ctx, {
			event: "tool_call",
			toolCallId: event.toolCallId,
			toolName: event.toolName,
			input: event.input,
			decision: "observe",
		});

		const config = driftConfig(ctx);
		if (!config.enabled) return undefined;
		if (!isTargetMutation(event, ctx.cwd, config.target)) return undefined;

		targetMutationSeen = true;
		await mirror(ctx, {
			event: "multifile_drift_target_mutation",
			toolCallId: event.toolCallId,
			toolName: event.toolName,
			input: event.input,
			details: {
				task: config.task,
				target: config.target,
				afterObservedRead: observedRead,
				afterDrift: driftFired,
			},
		});

		if (!observedRead) {
			await mirror(ctx, {
				event: "multifile_drift_invalid",
				toolCallId: event.toolCallId,
				toolName: event.toolName,
				input: event.input,
				reason: "target mutation before observed target read",
				details: { task: config.task, target: config.target },
			});
		} else if (!driftFired) {
			await mirror(ctx, {
				event: "multifile_drift_invalid",
				toolCallId: event.toolCallId,
				toolName: event.toolName,
				input: event.input,
				reason: "target mutation before drift fired",
				details: { task: config.task, target: config.target },
			});
		}
		return undefined;
	});

	pi.on("tool_result", async (event, ctx) => {
		await mirror(ctx, {
			event: event.isError ? "tool_error" : "tool_result",
			toolCallId: event.toolCallId,
			toolName: event.toolName,
			input: event.input,
			isError: event.isError,
			outputBytes: contentBytes(event.content),
			details: isRecord(event.details) ? event.details : undefined,
		});

		const config = driftConfig(ctx);
		if (!config.enabled || event.isError || observedRead) return undefined;
		if (!isTargetQueryResult(event, ctx.cwd, config.target)) return undefined;

		observedRead = true;
		await mirror(ctx, {
			event: "multifile_drift_observed_read",
			toolCallId: event.toolCallId,
			toolName: event.toolName,
			input: event.input,
			details: { task: config.task, target: config.target },
		});
		await fireDrift(ctx, config);
		return undefined;
	});

	pi.on("user_bash", async (event, ctx) => {
		await mirror(ctx, {
			event: "user_bash",
			toolName: "bash",
			input: {
				command: event.command,
				excludeFromContext: event.excludeFromContext,
			},
			decision: "observe",
		});
	});
}

function driftConfig(ctx) {
	const task = process.env.BENCH_MULTIFILE_DRIFT_TASK?.trim();
	const enabled = process.env.BENCH_MULTIFILE_DRIFT_ENABLED === "1" && !!task && !!SPECS[task];
	const spec = enabled ? SPECS[task] : undefined;
	return {
		enabled,
		task,
		target: spec?.target,
		old: spec?.old,
		new: spec?.new,
		root: ctx.cwd,
	};
}

async function fireDrift(ctx, config) {
	if (driftFired) return;
	const target = resolve(ctx.cwd, config.target);
	const state = resolve(ctx.cwd, `.${config.task}.drift-fired`);
	try {
		const text = await readText(target);
		let outcome = "fired";
		if (text.includes(config.old)) {
			await writeText(target, text.replace(config.old, config.new));
			await writeText(state, "fired\n");
		} else if (text.includes(config.new)) {
			outcome = "already-drifted";
			await writeText(state, "already-drifted\n");
		} else {
			throw new Error(`${config.target}: drift anchor missing`);
		}
		driftFired = true;
		await mirror(ctx, {
			event: "multifile_drift_fired",
			details: {
				task: config.task,
				target: config.target,
				outcome,
				targetMutationSeen,
			},
		});
	} catch (error) {
		await mirror(ctx, {
			event: "multifile_drift_error",
			reason: error instanceof Error ? error.message : String(error),
			details: { task: config.task, target: config.target },
		});
	}
}

function isTargetQueryResult(event, cwd, target) {
	if (event.toolName === "read") return pathMatches(cwd, target, event.input?.path);
	if (event.toolName !== "bash") return false;
	const command = event.input?.command;
	return typeof command === "string" && commandMentionsTarget(command, cwd, target) && bashLooksLikeQuery(command);
}

function isTargetMutation(event, cwd, target) {
	if (event.toolName === "edit" || event.toolName === "write") {
		return pathMatches(cwd, target, event.input?.path);
	}
	if (event.toolName !== "bash") return false;
	const command = event.input?.command;
	return typeof command === "string" && commandMentionsTarget(command, cwd, target) && bashLooksLikeMutation(command);
}

function pathMatches(cwd, target, rawPath) {
	if (typeof rawPath !== "string" || !rawPath.trim()) return false;
	const actual = normalize(isAbsolute(rawPath) ? rawPath : resolve(cwd, rawPath));
	const expected = normalize(resolve(cwd, target));
	return actual === expected;
}

function commandMentionsTarget(command, cwd, target) {
	const normalizedCommand = command.replace(/\\/g, "/");
	const normalizedTarget = target.replace(/\\/g, "/");
	const normalizedAbsoluteTarget = resolve(cwd, target).replace(/\\/g, "/");
	return normalizedCommand.includes(normalizedTarget) || normalizedCommand.includes(normalizedAbsoluteTarget);
}

function bashLooksLikeQuery(command) {
	const trimmed = command.trim();
	if (/^(cat|grep|awk|head|tail|wc)\b/.test(trimmed)) return true;
	if (/\bmd\s+(outline|blocks|block|section|search|tasks|stats|table|links|frontmatter)\b/.test(trimmed)) return true;
	return false;
}

function bashLooksLikeMutation(command) {
	const trimmed = command.trim();
	if (/\bmd\s+(replace-section|delete-section|replace-block|insert-block|delete-block|set|set-task|move-section)\b/.test(trimmed)) return true;
	if (/\bsed\b[^|;&]*\s-i\b/.test(trimmed)) return true;
	if (/\b(perl|ruby)\b[^|;&]*\s-pi\b/.test(trimmed)) return true;
	if (/\b(tee|mv|cp)\b/.test(trimmed)) return true;
	if (/(^|[^<])>{1,2}\s*['"]?[^'"\s]*mf0[12]\//.test(trimmed)) return true;
	return false;
}

function configuredLiveLogPath(cwd) {
	const raw = process.env.PI_AUDIT_LOG?.trim();
	if (!raw) return undefined;
	const expanded = raw === "~" ? process.env.HOME : raw.startsWith("~/") ? resolve(process.env.HOME ?? cwd, raw.slice(2)) : raw;
	return isAbsolute(expanded) ? expanded : resolve(cwd, expanded);
}

async function mirror(ctx, event) {
	const path = configuredLiveLogPath(ctx.cwd);
	if (!path) return;
	const info = sessionInfo(ctx);
	const fullEvent = {
		schema: AUDIT_SCHEMA,
		ts: new Date().toISOString(),
		source: "live",
		cwd: ctx.cwd,
		...info,
		...sanitizeEvent(event),
	};
	await appendJsonl(path, fullEvent);
}

function sessionInfo(ctx) {
	return {
		sessionId: ctx.sessionManager?.getSessionId?.(),
		sessionFile: ctx.sessionManager?.getSessionFile?.(),
	};
}

async function appendJsonl(path, event) {
	await mkdirAsync(dirname(path), { recursive: true });
	await appendFileAsync(path, `${JSON.stringify(event)}\n`, "utf8");
}

function sanitizeEvent(event) {
	const out = { ...event };
	if (out.input !== undefined) out.input = sanitizeForAudit(out.input);
	if (out.details !== undefined) out.details = sanitizeForAudit(out.details);
	return out;
}

function sanitizeForAudit(value, depth = 0, keyHint = "") {
	if (value == null) return value;
	if (typeof value === "string") return SECRET_KEY_RE.test(keyHint) ? "[REDACTED]" : truncate(value);
	if (typeof value === "number" || typeof value === "boolean") return value;
	if (depth >= 6) return "[MaxDepth]";
	if (Array.isArray(value)) return value.slice(0, 40).map((item) => sanitizeForAudit(item, depth + 1, keyHint));
	if (typeof value === "object") {
		const out = {};
		for (const [key, item] of Object.entries(value).slice(0, 80)) {
			out[key] = SECRET_KEY_RE.test(key) ? "[REDACTED]" : sanitizeForAudit(item, depth + 1, key);
		}
		return out;
	}
	return String(value);
}

function contentBytes(content) {
	if (!Array.isArray(content)) return 0;
	let total = 0;
	for (const item of content) {
		if (isRecord(item) && typeof item.text === "string") total += Buffer.byteLength(item.text, "utf8");
	}
	return total;
}

function truncate(value, maxLength = 1000) {
	return value.length <= maxLength ? value : `${value.slice(0, maxLength)}... [truncated ${value.length - maxLength} chars]`;
}

function isRecord(value) {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readText(path) {
	return new Promise((resolvePromise, reject) => {
		readFile(path, "utf8", (error, data) => (error ? reject(error) : resolvePromise(data)));
	});
}

function writeText(path, content) {
	return new Promise((resolvePromise, reject) => {
		writeFile(path, content, "utf8", (error) => (error ? reject(error) : resolvePromise()));
	});
}

function mkdirAsync(path, options) {
	return new Promise((resolvePromise, reject) => {
		mkdir(path, options, (error) => (error ? reject(error) : resolvePromise()));
	});
}

function appendFileAsync(path, content, encoding) {
	return new Promise((resolvePromise, reject) => {
		appendFile(path, content, encoding, (error) => (error ? reject(error) : resolvePromise()));
	});
}
