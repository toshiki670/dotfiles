from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable


@dataclass
class FailureRecord:
    file: str
    rule: str
    phase: str
    command: str
    exit_code: int


@dataclass
class LintContext:
    repo_root: Path
    tmp_dir: Path
    mode: str
    check_after_fix: bool
    verbose: bool
    summary: bool
    json_output: bool
    failures: list[FailureRecord] = field(default_factory=list)


@dataclass
class FileContext:
    rel_path: str
    abs_path: Path
    rendered_cache: dict[str, Path] = field(default_factory=dict)


@dataclass
class Rule:
    name: str
    match: Callable[[LintContext, FileContext], bool]
    fix: Callable[[LintContext, FileContext], int]
    check: Callable[[LintContext, FileContext], int]
