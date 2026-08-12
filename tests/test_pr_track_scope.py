from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_pr_track_scope", ROOT / "scripts" / "check_pr_track_scope.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
EXCEPTION_LABEL = MODULE.EXCEPTION_LABEL
check = MODULE.check


def event(body: str, labels: tuple[str, ...] = ()) -> dict[str, object]:
    return {
        "pull_request": {
            "body": body,
            "labels": [{"name": label} for label in labels],
        }
    }


def body(track: str, exception: str = "none", rationale: str = "none") -> str:
    return f"""- Conductor track: `{track}`
- Multi-track exception: `none`
- Exception tracks (only when absolutely inseparable): `{exception}`
- Why the work cannot be split (required for an exception): `{rationale}`
"""


def test_single_track_passes() -> None:
    receipt = check(event(body("10")), ["conductor/tracks/10-mcp-mvp/plan.md", "src/lib.rs"])
    assert receipt["status"] == "passed"


def test_single_track_rejects_another_track_path() -> None:
    receipt = check(
        event(body("10")),
        ["conductor/tracks/10-mcp-mvp/plan.md", "conductor/tracks/16-quality/plan.md"],
    )
    assert receipt["status"] == "failed"


def test_multi_requires_label_and_justification() -> None:
    receipt = check(event(body("MULTI", "10, 16", "too short")), [])
    assert receipt["status"] == "failed"
    assert len(receipt["errors"]) == 2


def test_necessary_multi_track_exception_passes() -> None:
    receipt = check(
        event(
            body(
                "MULTI",
                "10, 16",
                "The shared protocol baseline and its admission gate must change atomically.",
            ),
            (EXCEPTION_LABEL,),
        ),
        ["conductor/tracks/10-mcp-mvp/plan.md", "conductor/tracks/16-quality/plan.md"],
    )
    assert receipt["status"] == "passed"


def test_single_track_rejects_exception_label() -> None:
    receipt = check(event(body("10"), (EXCEPTION_LABEL,)), [])
    assert receipt["status"] == "failed"
