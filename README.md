Workspace which includes:
`vkcl` crate, which is the whole logic and API
`cli` which is a CLI wrapper around `api`

and yes i know this is something better done in-game (with something such a gecko code, or ideally, a pycore script) but i do not have the knowledge to do so

todo:
- rust version of `szs` and `brres` crates (`brres` is MUCH more complicated, probably will never try it)
  
### Credits
- Gabriela for the Blender Python script (whose logic i followed) and resources on manual visible SZS creation
- Epik95 for custom GOBJ models (`api/src/gobj`) and resources on manual visible SZS creation
- unfortunately, Ri*defi, for creating the `szs` and `brres` crates to decode Yaz0 and .brres files
  - i had to locally patch these crates (`szs-patched`, `brres-patched` and `brres-sys-patched`), since i had some bugs with building them, since they actually are a C++ wrapper