Workspace which includes:<br/>
`vkcl` core logic and main API<br/>
`vkcl-cli` a simple CLI wrapper<br/>
`vkcl-py` python bindings

and yes i know this is something better done in-game (with something such a gecko code, or ideally, a pycore script) but i do not have the knowledge to do so

todo:
- rust version of `szs` and `brres` crates (`brres` is MUCH more complicated, probably will never try it)

## Usage

### CLI
download `vkcl-cli.zip` from the releases tab, then rename the executable for your OS to `vkcl` (or `vkcl.exe` on Windows) so you can run it simply<br/>
```bash
vkcl replace beginner_course.szs
vkcl overlay beginner_course.szs --ckpt --inv_walls
```
to run globally, add the folder to your env variables PATH

### Python
download `vkcl-py.zip` from the releases tab and install the wheel for your OS:
```bash
pip install <your_os_file>.whl
```
example usage:
```python
import vkcl_py

# visible collision with highlighted horizontal walls and item road
highlight = vkcl_py.HighlightOption(horizontal_wall=True)
special = vkcl_py.SpecialPlanesOption(item_road=True)
overlay = vkcl_py.OverlayOption()
vkcl_py.replace("beginner_course.szs", "beginner_course.szs", highlight, special, overlay)

# 2D map with wireframe and GOBJ points
kcl = vkcl_py.KclDrawOptions(wireframe=True)
kmp = vkcl_py.KmpDrawOptions(gobj=True)
vkcl_py.draw("beginner_course.szs", "output.png", kcl, kmp)
```
  
## Credits
- **Gabriela** for the Blender Python script (whose logic i followed) and resources on manual visible SZS creation
- **Epik95** for custom GOBJ models (`vkcl/src/gobj`) and resources on manual visible SZS creation
- unfortunately, Ri*defi, for creating the `szs` and `brres` crates to decode Yaz0 and .brres files
  - i had to locally patch these crates (`szs-patched`, `brres-patched` and `brres-sys-patched`), since i had some bugs with building them, since they actually are a C++ wrapper
