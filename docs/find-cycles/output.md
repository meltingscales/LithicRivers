make find-cycles ^C
melty@ubuntu-slimdragon:~/Git/LithicRivers$ make find-cycles 
🧹 Cleaning build artifacts...
rm -rf build/ dist/ *.egg-info/ .coverage htmlcov/ coverage/
rm -f *.log
rm -f coverage.lcov
find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
find . -type f -name "*.pyc" -delete 2>/dev/null || true
rm -rf .pytest_cache/ .mypy_cache/ .ruff_cache/
rm -rf lithicrivers-test-saves/
rm -rf *.speedscope
✅ Clean complete!
uv run --extra cycles python lithicrivers/scripts/find_object_cycles.py
/home/melty/Git/LithicRivers/.venv/lib/python3.12/site-packages/steam/steamid.py:504: SyntaxWarning: invalid escape sequence '\-'
  if not re.match(r'^['+_csgofrcode_chars+'\-]{10}$', code):
/home/melty/Git/LithicRivers/.venv/lib/python3.12/site-packages/steam/steamid.py:577: SyntaxWarning: invalid escape sequence '\('
  data_match = re.search("OpenGroupChat\( *'(?P<steamid>\d+)'", text)

=== LithicRivers Object Graph Cycle Analysis ===

{'cache_size': 111180,
 'empty_chunks': 9,
 'tile_distribution': {Tile(tileid='Bone Block', sprite_sheet=['|', '||\n||', '|||\n|||\n|||'], description=None, drops={0.7: Item(name='Rock', sprite_sheet=['*', '**\n**', '***\n***\n***']), 0.3: Item(name='Gold Nugget', sprite_sheet=['c', 'cc\ncc', 'ccc\nccc\nccc'])}): 100,
                       Tile(tileid='Door', sprite_sheet=['D', 'DD\nDD', 'DDD\nDDD\nDDD'], description=None, drops={0.5: Item(name='Rock', sprite_sheet=['*', '**\n**', '***\n***\n***'])}): 100,
                       Tile(tileid='Iron Scrap', sprite_sheet=['=', '==\n==', '===\n===\n==='], description=None, drops={0.8: Item(name='Iron Scrap', sprite_sheet=['=', '==\n==', '===\n===\n===']), 0.2: Item(name='Gold Nugget', sprite_sheet=['c', 'cc\ncc', 'ccc\nccc\nccc'])}): 100,
                       Tile(tileid='buried_treasure', sprite_sheet=['$', '$$\n$$', '$$$\n$$$\n$$$'], description=None, drops={0.3: Item(name='Gold Nugget', sprite_sheet=['c', 'cc\ncc', 'ccc\nccc\nccc']), 0.7: Item(name='Diamond', sprite_sheet=['d', 'dd\ndd', 'ddd\nddd\nddd'])}): 100,
                       Tile(tileid='Scrap Electronics', sprite_sheet=['e', 'ee\nee', 'eee\neee\neee'], description=None, drops={0.6: Item(name='Scrap Electronics', sprite_sheet=['e', 'ee\nee', 'eee\neee\neee']), 0.4: Item(name='Gold Nugget', sprite_sheet=['c', 'cc\ncc', 'ccc\nccc\nccc'])}): 100,
                       Tile(tileid='Bedrock', sprite_sheet=['#', '|/\n/|', '|,/\n/|\\\n|/|'], description=None, drops=None): 100,
                       Tile(tileid='Empty', sprite_sheet=[' ', '  \n  ', '   \n   \n   '], description=None, drops=None): 100,
                       Tile(tileid='Gold Ore', sprite_sheet=['?', '??\n??', '???\n???\n???'], description=None, drops={0.9: Item(name='Gold Nugget', sprite_sheet=['c', 'cc\ncc', 'ccc\nccc\nccc']), 0.1: Item(name='Diamond', sprite_sheet=['d', 'dd\ndd', 'ddd\nddd\nddd'])}): 100,
                       Tile(tileid='Tree', sprite_sheet=['t', '/\\\n||', '/|\\\n;|;\n/|\\'], description=None, drops={0.5: Item(name='Stick', sprite_sheet=[' ']), 0.3: Item(name='Log', sprite_sheet=['|', '||\n||', '|||\n|||\n|||']), 0.2: Item(name='Acorn', sprite_sheet=['o', 'oo\noo', 'ooo\nooo\nooo'])}): 100,
                       Tile(tileid='Dirt', sprite_sheet=[',', ',.\n.,', ',.,\n.,.\n,.,'], description=None, drops={0.99: Item(name='Rock', sprite_sheet=['*', '**\n**', '***\n***\n***']), 0.01: Item(name='Gold Nugget', sprite_sheet=['c', 'cc\ncc', 'ccc\nccc\nccc'])}): 100,
                       Tile(tileid='Cloud', sprite_sheet=['~', '~o\noo', '.~~\n~~o\n~oo'], description=None, drops=None): 100},
 'total_chunks': 27,
 'total_tile_types': 100,
 'used_chunks': 18}

[3] Objgraph cycle detection across key components (World, Player, Engine, WorldData)...
- [1/16] Searching chain from game -> Game ...
  No chain from game to Game within max_depth. Traversals: 1003095              
- [2/16] Searching chain from world -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-zog2n0hx.dot (2 nodes)
Image generated as cycle-world-to-game.png
  Found. Wrote cycle-world-to-game.png. Traversals: 2
- [3/16] Searching chain from player -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-xdxh2ppm.dot (2 nodes)
Image generated as cycle-player-to-game.png
  Found. Wrote cycle-player-to-game.png. Traversals: 3
- [4/16] Searching chain from world_data -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-25kzb1jv.dot (3 nodes)
Image generated as cycle-world_data-to-game.png
  Found. Wrote cycle-world_data-to-game.png. Traversals: 9
- [5/16] Searching chain from message_log -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-bk1by742.dot (2 nodes)
Image generated as cycle-message_log-to-game.png
  Found. Wrote cycle-message_log-to-game.png. Traversals: 3
- [6/16] Searching chain from save_manager -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-ixhn9kh3.dot (2 nodes)
Image generated as cycle-save_manager-to-game.png
  Found. Wrote cycle-save_manager-to-game.png. Traversals: 2
- [7/16] Searching chain from game.message_log -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-fves5p0q.dot (2 nodes)
Image generated as cycle-game.message_log-to-game.png
  Found. Wrote cycle-game.message_log-to-game.png. Traversals: 3
- [8/16] Searching chain from game.player -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-nq4fdfyv.dot (2 nodes)
Image generated as cycle-game.player-to-game.png
  Found. Wrote cycle-game.player-to-game.png. Traversals: 2
- [9/16] Searching chain from game.save_manager -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-c2zmtuxl.dot (2 nodes)
Image generated as cycle-game.save_manager-to-game.png
  Found. Wrote cycle-game.save_manager-to-game.png. Traversals: 3
- [10/16] Searching chain from game.viewport -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-cikypydx.dot (2 nodes)
Image generated as cycle-game.viewport-to-game.png
  Found. Wrote cycle-game.viewport-to-game.png. Traversals: 4
- [11/16] Searching chain from game.world -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-tp2bzzla.dot (2 nodes)
Image generated as cycle-game.world-to-game.png
  Found. Wrote cycle-game.world-to-game.png. Traversals: 3
- [12/16] Searching chain from world.data -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-oqariqp1.dot (3 nodes)
Image generated as cycle-world.data-to-game.png
  Found. Wrote cycle-world.data-to-game.png. Traversals: 9
- [13/16] Searching chain from world.fluid_manager -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-y0uxc4pq.dot (3 nodes)
Image generated as cycle-world.fluid_manager-to-game.png
  Found. Wrote cycle-world.fluid_manager-to-game.png. Traversals: 14
- [14/16] Searching chain from world.generator -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-txy2l0l9.dot (3 nodes)
Image generated as cycle-world.generator-to-game.png
  Found. Wrote cycle-world.generator-to-game.png. Traversals: 10
- [15/16] Searching chain from player.position -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-eladdboj.dot (3 nodes)
Image generated as cycle-player.position-to-game.png
  Found. Wrote cycle-player.position-to-game.png. Traversals: 116
- [16/16] Searching chain from world_data.world_generator -> Game ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-al649rm6.dot (3 nodes)
Image generated as cycle-world_data.world_generator-to-game.png
  Found. Wrote cycle-world_data.world_generator-to-game.png. Traversals: 10
- [1/16] Searching chain from Game -> game ...
  No chain from Game to game within max_depth. Traversals: 1003102              
- [2/16] Searching chain from Game -> world ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-fat8ji5j.dot (3 nodes)
Image generated as cycle-game-to-world.png
  Found. Wrote cycle-game-to-world.png. Traversals: 388665
- [3/16] Searching chain from Game -> player ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-_mjea7ad.dot (3 nodes)
Image generated as cycle-game-to-player.png
  Found. Wrote cycle-game-to-player.png. Traversals: 271490
- [4/16] Searching chain from Game -> world_data ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-alzcd3x0.dot (3 nodes)
Image generated as cycle-game-to-world_data.png
  Found. Wrote cycle-game-to-world_data.png. Traversals: 271512
- [5/16] Searching chain from Game -> message_log ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-bqqgsyg3.dot (3 nodes)
Image generated as cycle-game-to-message_log.png
  Found. Wrote cycle-game-to-message_log.png. Traversals: 271497
- [6/16] Searching chain from Game -> save_manager ...
  No chain from Game to save_manager within max_depth. Traversals: 1003088      
- [7/16] Searching chain from Game -> game.message_log ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-8z12tnzg.dot (3 nodes)
Image generated as cycle-game-to-game.message_log.png
  Found. Wrote cycle-game-to-game.message_log.png. Traversals: 389057
- [8/16] Searching chain from Game -> game.player ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-u5wwkk47.dot (3 nodes)
Image generated as cycle-game-to-game.player.png
  Found. Wrote cycle-game-to-game.player.png. Traversals: 271482
- [9/16] Searching chain from Game -> game.save_manager ...
  No chain from Game to game.save_manager within max_depth. Traversals: 1003091 
- [10/16] Searching chain from Game -> game.viewport ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-tom00y2m.dot (3 nodes)
Image generated as cycle-game-to-game.viewport.png
  Found. Wrote cycle-game-to-game.viewport.png. Traversals: 658689
- [11/16] Searching chain from Game -> game.world ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-s3nxt02x.dot (3 nodes)
Image generated as cycle-game-to-game.world.png
  Found. Wrote cycle-game-to-game.world.png. Traversals: 271548
- [12/16] Searching chain from Game -> world.data ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-ump16eiw.dot (3 nodes)
Image generated as cycle-game-to-world.data.png
  Found. Wrote cycle-game-to-world.data.png. Traversals: 271511
- [13/16] Searching chain from Game -> world.fluid_manager ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-z0vol4a0.dot (3 nodes)
Image generated as cycle-game-to-world.fluid_manager.png
  Found. Wrote cycle-game-to-world.fluid_manager.png. Traversals: 326279
- [14/16] Searching chain from Game -> world.generator ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-o3z68fuk.dot (4 nodes)
Image generated as cycle-game-to-world.generator.png
  Found. Wrote cycle-game-to-world.generator.png. Traversals: 277407
- [15/16] Searching chain from Game -> player.position ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-g_cpdgji.dot (3 nodes)
Image generated as cycle-game-to-player.position.png
  Found. Wrote cycle-game-to-player.position.png. Traversals: 346599
- [16/16] Searching chain from Game -> world_data.world_generator ...
  Rendering chain PNG...                                                        
Graph written to /tmp/objgraph-x1_jxhaa.dot (4 nodes)
Image generated as cycle-game-to-world_data.world_generator.png
  Found. Wrote cycle-game-to-world_data.world_generator.png. Traversals: 277406

Done. Check generated PNGs (cycle-*, game-backrefs.png).

[4] Backrefs to Game (graph). This can be noisy; consider installing graphviz.)
  show_backrefs(Game) -  Graph written to /tmp/objgraph-carpe_h3.dot (24 nodes)
  show_backrefs(Game) |  Image generated as game-backrefs.png
  Wrote game-backrefs.png                                                       

[5] Objgraph type stats (top 20)...
Tile                       111197
function                   31038
cell                       28192
tuple                      23938
dict                       17448
list                       13073
Item                       12183
member_descriptor          4817
ReferenceType              2806
getset_descriptor          2098
FieldDescriptor            1867
_FieldProperty             1862
EMsg                       1855
wrapper_descriptor         1630
type                       1397
method_descriptor          1327
builtin_function_or_method 1248
staticmethod               1140
property                   791
module                     509

Done. Check generated PNGs (cycle-*, game-backrefs.png).
