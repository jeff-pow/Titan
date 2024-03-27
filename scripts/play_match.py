# Credit to Algorhythm-sxv for this
import chess
import chess.engine
import chess.pgn

import logging
logging.basicConfig(level=logging.DEBUG)

pgn = open("error_base.pgn")
game = chess.pgn.read_game(pgn)
dev = chess.engine.SimpleEngine.popen_uci("../Titan", debug=True)
dev.configure({"Threads": 1, "Hash": 16})
base = chess.engine.SimpleEngine.popen_uci("../main", debug=True)
base.configure({"Threads": 1, "Hash": 16})

board = game.board()
for node in game.mainline():
    comment = node.comment
    node_count = int(comment.split(' ')[3].rstrip(','))
    color = not node.turn()
    engine_to_play = dev
    if color == chess.WHITE:
        engine_to_play = base
    result = engine_to_play.play(board, chess.engine.Limit(nodes=node_count))
    print(node_count, node.move, result.move)
    assert (result.move == node.move)
    board.push(result.move)

result = base.play(board, chess.engine.Limit(time=0.01))

print(result)

base.quit()
dev.quit()
