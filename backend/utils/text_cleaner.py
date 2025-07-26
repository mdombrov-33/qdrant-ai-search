import re
import unicodedata


def clean_text(text: str, lowercase: bool = False) -> str:
    """
    Clean and normalize input text.
    - Normalize unicode to NFKC form
    - Remove control/non-printable characters
    - Collapse multiple whitespace into single space
    - Optionally lowercase text
    """
    # Unicode normalization
    text = unicodedata.normalize("NFKC", text)

    # Remove control characters (except newline)
    text = "".join(ch for ch in text if ch.isprintable() or ch == "\n")

    # Replace newlines, tabs, multiple spaces with single space
    text = re.sub(r"\s+", " ", text)

    if lowercase:
        text = text.lower()

    return text.strip()
