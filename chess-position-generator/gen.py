import chess
import chess.pgn
import json
import random
import subprocess


def random_position(max_plies=30):
    board = chess.Board()
    plies = random.randint(1, max_plies)
    for _ in range(plies):
        if board.is_game_over():
            break
        move = random.choice(list(board.legal_moves))
        board.push(move)
    return board


def stockfish_perft(fen, depth, p):
    if depth == 0:
        return 1

    p.stdin.write(bytes(f"position fen {fen}\n", encoding='utf8'))
    p.stdin.write(bytes(f"go perft {depth}\n", encoding='utf8'))
    p.stdin.flush()

    result = {}
    while True:
        line = p.stdout.readline().strip().decode("utf8")
        if not line:
            break
        if ":" in line:  # e.g. "a2a3: 20"
            move, nodes = line.split(":")
            result[move.strip()] = int(nodes.strip())

    return result


def perft(fen, depth, p):
    perfts = {}

    for depth in range(1, depth + 1):
        perfts[depth] = stockfish_perft(fen, depth, p)
        for _ in range(2):
            p.stdout.readline().strip().decode("utf8")

    return perfts


p = subprocess.Popen(
    "stockfish", stdin=subprocess.PIPE, stdout=subprocess.PIPE)

p.stdin.write(b"uci\n")
p.stdin.flush()

for i in range(4):
    p.stdout.readline().strip().decode("utf8")

positions = []
for i in range(10):
    b = random_position()
    fen = b.fen()

    depths = perft(fen, 6, p)
    positions.append({"fen": fen, "depths": depths})

with open("perft_dataset_depth_6.json", "w") as f:
    json.dump(positions, f, indent=2)

p.stdin.write(b"quit\n")
p.stdin.flush()
p.terminate()
