### v0.2.1 [May 30, 2020]
* Added basic UI support
    * Currently in Alpha phase
* Fixed the windowing issue for the main world renderer

### v0.2.0 [May 20, 2020]
* Chunks now optimized with transparent blocks
* Added [T] key to escape the mouse lock (temp.)

### v0.1.0 [May 18, 2020]
* World is now rendered by chunks instead of each discrete block type.
* Each chunk owns a block data, which gets converted into vertices and indices for rendering
    * a lot of GPU optimization here: index rendering, smaller vertex data types, etc.
* Each chunk has its own block data, which gets called to render in conjunction with the parameter of the owner of mesh struct.
