import re


def seconds_to_mmss(seconds):
    minutes = int(seconds // 60)
    secs = int(round(seconds % 60))
    return f"{minutes}:{secs:02d}"


def convert_pgn_clocks(input_file, output_file):
    with open(input_file, "r") as f:
        pgn = f.read()

    # Replace tl=XX.XXXs in comments with [%clk mm:ss]
    def repl(match):
        tl_sec = float(match.group(1))
        return f"{{[%%clk {seconds_to_mmss(tl_sec)}]"

    # regex: find tl=number (with optional decimal) inside {...}
    new_pgn = re.sub(r"{.*?tl=(\d+\.?\d*)s", repl, pgn)

    with open(output_file, "w") as f:
        f.write(new_pgn)


if __name__ == "__main__":
    convert_pgn_clocks("log.pgn", "log_chesscom.pgn")
