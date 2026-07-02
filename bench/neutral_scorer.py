from __future__ import annotations

from markdown_it import MarkdownIt


_NEUTRAL_MD = MarkdownIt("commonmark", {"html": True}).enable(["table"])


def _render_inline_to_plaintext(children) -> str:
    """Concatenate text-bearing inline children, dropping markup wrappers."""
    parts = []
    for child in children or []:
        if child.type in ("text", "code_inline"):
            parts.append(child.content)
        elif child.type == "image":
            parts.append(_render_inline_to_plaintext(child.children))
        elif child.type in ("softbreak", "hardbreak"):
            parts.append(" ")
    return "".join(parts)


def neutral_heading_tree(content: str) -> list[tuple[int, str]]:
    """Extract heading tree using markdown-it-py, independent of the md binary."""
    tokens = _NEUTRAL_MD.parse(content)
    tree = []
    for i, tok in enumerate(tokens):
        if tok.type == "heading_open":
            level = int(tok.tag[1:])
            if i + 1 < len(tokens) and tokens[i + 1].type == "inline":
                text = _render_inline_to_plaintext(tokens[i + 1].children)
            else:
                text = ""
            tree.append((level, text))
    return tree


def neutral_block_texts(content: str) -> list[str]:
    """Extract source-faithful block-level text using markdown-it-py."""
    tokens = _NEUTRAL_MD.parse(content)
    lines = content.splitlines()
    texts = []
    i = 0
    while i < len(tokens):
        tok = tokens[i]
        if tok.type in (
            "heading_open",
            "paragraph_open",
            "bullet_list_open",
            "ordered_list_open",
            "blockquote_open",
            "table_open",
            "html_block",
            "code_block",
            "fence",
            "hr",
        ):
            if tok.type == "hr":
                if tok.map is not None:
                    start, end = tok.map
                    texts.append("\n".join(lines[start:end]).strip())
                else:
                    texts.append("---")
                i += 1
                continue
            if tok.type == "heading_open":
                if tok.map is not None:
                    start, end = tok.map
                    texts.append("\n".join(lines[start:end]).strip())
                elif i + 1 < len(tokens) and tokens[i + 1].type == "inline":
                    texts.append((tokens[i + 1].content or "").strip())
                while i < len(tokens) and tokens[i].type != "heading_close":
                    i += 1
                i += 1
                continue
            if tok.type in ("html_block", "code_block", "fence"):
                texts.append((tok.content or "").strip())
                i += 1
                continue
            if tok.type in (
                "paragraph_open",
                "bullet_list_open",
                "ordered_list_open",
                "blockquote_open",
                "table_open",
            ) and tok.map is not None:
                start, end = tok.map
                texts.append("\n".join(lines[start:end]).strip())
                close_type = tok.type.replace("_open", "_close")
                depth = 1
                i += 1
                while i < len(tokens) and depth > 0:
                    t = tokens[i]
                    if t.type == tok.type:
                        depth += 1
                    elif t.type == close_type:
                        depth -= 1
                        if depth == 0:
                            i += 1
                            break
                    i += 1
                continue
            close_type = tok.type.replace("_open", "_close")
            depth = 1
            parts = []
            i += 1
            while i < len(tokens) and depth > 0:
                t = tokens[i]
                if t.type == tok.type:
                    depth += 1
                elif t.type == close_type:
                    depth -= 1
                    if depth == 0:
                        break
                if t.type == "inline" and t.content:
                    parts.append(t.content)
                elif t.type in ("code_block", "fence") and t.content:
                    parts.append(t.content.strip())
                i += 1
            texts.append("\n".join(parts).strip())
        i += 1
    return [text for text in texts if text]
