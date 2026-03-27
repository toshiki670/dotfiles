from lint.cli import build_parser


def test_cli_defaults() -> None:
    args = build_parser().parse_args(["check"])
    assert args.mode == "check"
    assert args.check_after_fix == "on"
    assert args.summary is False
    assert args.json_output is False
    assert args.verbose is False
