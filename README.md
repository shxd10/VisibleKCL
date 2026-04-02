currently a CLI tool, thought for TAS comp, that quickly transforms a normal `.szs` file in a `.szs` file but with visible collision, checkpoints, and many other customization options (e.g. highlighting horizontal walls). Along with a `draw` command for 2d maps
<br/>
<br/>
and yes i know this is something better done in-game (with something such a gecko code, or ideally, a pycore script) but i do not have the knowledge to do so<br/>

todo:
- rust version of `szs` and `brres` crates (`brres` is MUCH more complicated, probably will never try it)
  
### Credits
- Ri*defi, for creating the `szs` and `brres` crates to decode Yaz0 and .brres files
  - i had to locally patch these crates though (`szs-patched`, `brres-patched` and `brres-sys-patched`), since i had some bugs with building them, since they actually are a C++ wrapper
- Epik95 and Gabriela for the manual visible collision tutorial and resources, most importantly the custom GOBJ models (epik95, `src/gobj`) and the blender python script (gabriela)