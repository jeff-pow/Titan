import chess
import chess.engine
import chess.pgn

import logging
logging.basicConfig(level=logging.DEBUG)

pgn = open("game.pgn")
game = chess.pgn.read_game(pgn)
base = chess.engine.SimpleEngine.popen_uci("./titan", debug=True)
base.configure({"Threads": 1, "Hash": 16})

# Determine which side the engine is playing
if game.headers["White"] == "titan":
    offset = 0
else:
    offset = 1

board = game.board()

# Iterate over each move in the mainline of the game
for idx, node in enumerate(game.mainline()):
    # Alternate between player's and engine's moves based on the offset
    if (idx % 2) == offset:
        comment = node.comment
        print(node.comment)

        node_count = int(comment.split(" ")[-1].rstrip(","))

        result = base.play(board, chess.engine.Limit(nodes=node_count))

        assert result.move == node.move, "Engine move does not match PGN move"
        board.push(result.move)

    else:
        board.push(node.move)

# Perform one final move from the engine after the game ends
result = base.play(board, chess.engine.Limit(time=5.01))
print(result)

base.quit()
