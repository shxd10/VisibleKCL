A crate for transforming `.szs` files, replace or overlay course models with visible collision, checkpoints, horizontal walls and more
### Usage

Replacing a course model with KCL collision data, while highlighting horizontal walls and displaying item road:

```rust
use vkcl::{HighlightOption, SpecialPlanesOption, OverlayOption};

vkcl::replace(
    "beginner_course.szs",
    "beginner_course_visible.szs",
    &HighlightOption { horizontal_wall: true, ..Default::default() },
    &SpecialPlanesOption { item_road: true, ..Default::default() },
    &OverlayOption::default(),
    false,
)?;
```

Overlaying checkpoint lines and invisible walls on top of the original course model:

```rust
use vkcl::OverlayOption;

vkcl::overlay(
    "beginner_course.szs",
    "beginner_course_visible.szs",
    &OverlayOption { ckpt: true, ckpt_side: true, inv_walls: true, ..Default::default() },
    false,
)?;
```