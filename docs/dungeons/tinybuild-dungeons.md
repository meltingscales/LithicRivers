Procedural Dungeon Generation Algorithm Explained
So today I'm going to be a little different and talk about one technical aspect of my game TinyKeep, that is random procedural dungeon generation. It's pretty over-engineered, but hopefully will give anyone interested some ideas on generating dungeon layouts for their own games.

The interactive demo can be found here: Dungeon Generation Demo

Here's how I do it, step by step:

1 . First I set the number of cells I want to generate, say 150. This is an arbitrary amount really, but the higher the number the larger the dungeon and in general more complexity.

2 . For each "cell" I spawn a Rectangle of random width and length within some radius. Again the radius doesn't matter too much, but it should probably be proportionate to the number of cells.

Instead of using uniformly distributed random numbers (the default Math.random generator in most languages), I'm using Park-Miller Normal Distribution. This skews the size of the cells so that they are more likely to be of a small size (more smaller cells, less larger cells). The reason for this will be explained later!

In addition to this I ensure that the ratio between the width and length of each cell is not too large, we don't want perfectly square rooms but neither do we want really skinny ones, but somewhere in between.

3 . At this point we have 150 random cells in a small area, most are overlapping. Next we use simple separation steering behaviour to separate out all of the rectangles so that none are overlapping. This technique ensures that the cells are not overlapping, yet in general remain as tightly packed together as possible.

4 . We fill in any gaps with 1x1 sized cells. The result is that we end up with a square grid of differently sized cells, all perfectly packed together.

5 . Here is where the fun begins. We determine which of the cells in the grid are rooms - any cell with a width and height above a certain threshold is made into a room. Because of the Park-Miller Normal Distribution described earlier, there will only be a small amount of rooms in comparison to the number of cells, with lots of space between. The remaining cells are still useful however... read on.

6 . For the next stage we want to link each room together. To begin we construct a graph of all of the rooms' center points using Delaunay Triangulation. So now all rooms are connected to each other without intersecting lines.

7 . Because we don't want every single room to be linked to every other with a corridor (that would make for a very confusing layout), we then construct a Minimal Spanning Tree using the previous graph. This creates a graph that guarantees all rooms are connected (and therefore reachable in the game).

8 . The minimal spanning tree looks nice, but again is a boring dungeon layout because it contains no loops, it is the other extreme to the Delaunay Triangulation. So now we re-incorporate a small number of edges from the triangulated graph (say 15% of the remaining edges after the minimal spanning tree has been created). The final layout will therefore be a graph of all rooms, each guaranteed to be reachable and containing some loops for variety.

9 . To convert the graph to corridors, for each edge we construct a series of straight lines (or L shapes) going from each room of the graph to its neighbour. This is where the cells we have not yet used (those cells in the grid which are not rooms) become useful. Any cells which intersect with the L shapes become corridor tiles. Because of the variety in cell sizes, the walls of the corridors will be twisty and uneven, perfect for a dungeon.

And here's an example of the finished result!

Example screenshot

Thanks, hope you guys enjoy :)

The algorithm is used for our upcoming 3D dungeon crawler TinyKeep, currently in development:

TinyKeep - A 3D Multiplayer Dungeon Crawler with Frighteningly Intelligent Monster AI


