# File Structure
| File Name | Purpose |
|-----------|---------|
| block.rs | Loading each (low-level) block pertaining in the world to be rendered (e.g. flowers, actual blocks, multi-block doors) |
| camera.rs | The camera (where the MVP matrix exists) for rendering the world and for translation |
| chunk.rs | The struct for holding chunk datas: block datas, position |
| datatype.rs* | A file for holding all the data struct types for consistency and uniformity of types |
| main.rs | Setup and the main rendering loop |
| renderer.rs | A struct for holding all the rendering information to be rendered |
| player.rs**  | Holds camera struct and pertains inventory, effects on the player information  |
| terrain.rs | A struct for holding the instances of blocks to be readily available when generating the terrain |
| texture.rs | A texture manager for specific types of meshes |
| world.rs | An instance to hold all the chunks; gets loaded when the player instantiates or loads worlds |

\* Yet to be integrated  
\** Planned  

### mesh
Contains all the mesh rendering for the world.

### ui
The basic GUI for the menus and game.

### hud (planned)
Basic HUD for any GUI that requires game context 
(e.g. Hotbar, Inventory, HP, ...)
