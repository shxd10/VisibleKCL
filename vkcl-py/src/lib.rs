use pyo3::prelude::*;

#[pymodule]
mod vkcl_py {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // rewriting the structs

    #[pyclass]
    #[derive(Default)]
    pub struct HighlightOption {
        #[pyo3(get, set)]
        pub soft_wall: bool,
        #[pyo3(get, set)]
        pub horizontal_wall: bool,
    }

    #[pymethods]
    impl HighlightOption {
        #[new]
        #[pyo3(signature = (soft_wall=false, horizontal_wall=false))]
        fn new(soft_wall: bool, horizontal_wall: bool) -> Self {
            Self {
                soft_wall,
                horizontal_wall,
            }
        }
    }

    #[pyclass]
    #[derive(Default)]
    pub struct SpecialPlanesOption {
        #[pyo3(get, set)]
        pub item_road: bool,
        #[pyo3(get, set)]
        pub item_wall: bool,
        #[pyo3(get, set)]
        pub force_recalc: bool,
        #[pyo3(get, set)]
        pub sound_trigger: bool,
        #[pyo3(get, set)]
        pub effect_trigger: bool,
        #[pyo3(get, set)]
        pub item_state_modifier: bool,
    }

    #[pymethods]
    impl SpecialPlanesOption {
        #[new]
        #[pyo3(signature = (
            item_road=false,
            item_wall=false,
            force_recalc=false,
            sound_trigger=false,
            effect_trigger=false,
            item_state_modifier=false,
        ))]
        fn new(
            item_road: bool,
            item_wall: bool,
            force_recalc: bool,
            sound_trigger: bool,
            effect_trigger: bool,
            item_state_modifier: bool,
        ) -> Self {
            Self {
                item_road,
                item_wall,
                force_recalc,
                sound_trigger,
                effect_trigger,
                item_state_modifier,
            }
        }
    }

    #[pyclass]
    #[derive(Default)]
    pub struct OverlayOption {
        #[pyo3(get, set)]
        pub ckpt: bool,
        #[pyo3(get, set)]
        pub ckpt_side: bool,
        #[pyo3(get, set)]
        pub inv_walls: bool,
        #[pyo3(get, set)]
        pub gobj: bool,
    }

    #[pymethods]
    impl OverlayOption {
        #[new]
        #[pyo3(signature = (ckpt=false, ckpt_side=false, inv_walls=false, gobj=false))]
        fn new(ckpt: bool, ckpt_side: bool, inv_walls: bool, gobj: bool) -> Self {
            Self {
                ckpt,
                ckpt_side,
                inv_walls,
                gobj,
            }
        }
    }

    // LONG draw options

    #[pyclass]
    #[derive(Default)]
    pub struct KclDrawOptions {
        #[pyo3(get, set)]
        pub wireframe: bool,
        #[pyo3(get, set)]
        pub shading: f32,
    }

    #[pymethods]
    impl KclDrawOptions {
        #[new]
        #[pyo3(signature = (wireframe=false, shading=0.5))]
        fn new(wireframe: bool, shading: f32) -> Self {
            Self { wireframe, shading }
        }
    }

    #[pyclass]
    #[derive(Default)]
    pub struct KmpDrawOptions {
        #[pyo3(get, set)]
        pub thickness: u32,
        #[pyo3(get, set)]
        pub ktpt: bool,
        #[pyo3(get, set)]
        pub enpt: bool,
        #[pyo3(get, set)]
        pub itpt: bool,
        #[pyo3(get, set)]
        pub ckpt: bool,
        #[pyo3(get, set)]
        pub ckpt_side_lines: bool,
        #[pyo3(get, set)]
        pub gobj: bool,
        #[pyo3(get, set)]
        pub poti: bool,
        #[pyo3(get, set)]
        pub area: bool,
        #[pyo3(get, set)]
        pub came: bool,
        #[pyo3(get, set)]
        pub jgpt: bool,
        #[pyo3(get, set)]
        pub jgpt_lines: bool,
        #[pyo3(get, set)]
        pub cnpt: bool,
        #[pyo3(get, set)]
        pub mspt: bool,
        #[pyo3(get, set)]
        pub stgi: bool,
    }

    #[pymethods]
    impl KmpDrawOptions {
        #[new]
        #[pyo3(signature = (
            thickness=8,
            ktpt=false,
            enpt=false,
            itpt=false,
            ckpt=true,
            ckpt_side_lines=true,
            gobj=false,
            poti=false,
            area=false,
            came=false,
            jgpt=true,
            jgpt_lines=true,
            cnpt=false,
            mspt=false,
            stgi=false,
        ))]
        fn new(
            thickness: u32,
            ktpt: bool,
            enpt: bool,
            itpt: bool,
            ckpt: bool,
            ckpt_side_lines: bool,
            gobj: bool,
            poti: bool,
            area: bool,
            came: bool,
            jgpt: bool,
            jgpt_lines: bool,
            cnpt: bool,
            mspt: bool,
            stgi: bool,
        ) -> Self {
            Self {
                thickness,
                ktpt,
                enpt,
                itpt,
                ckpt,
                ckpt_side_lines,
                gobj,
                poti,
                area,
                came,
                jgpt,
                jgpt_lines,
                cnpt,
                mspt,
                stgi,
            }
        }
    }

    // finally, the functions

    #[pyfunction]
    #[pyo3(signature = (input_path, output_path, highlight, special, overlay, write_obj=false))]
    fn replace(
        input_path: &str,
        output_path: &str,
        highlight: &HighlightOption,
        special: &SpecialPlanesOption,
        overlay: &OverlayOption,
        write_obj: bool,
    ) -> PyResult<()> {
        vkcl::replace(
            input_path,
            output_path,
            &vkcl::HighlightOption {
                soft_wall: highlight.soft_wall,
                horizontal_wall: highlight.horizontal_wall,
            },
            &vkcl::SpecialPlanesOption {
                item_road: special.item_road,
                item_wall: special.item_wall,
                force_recalc: special.force_recalc,
                sound_trigger: special.sound_trigger,
                effect_trigger: special.effect_trigger,
                item_state_modifier: special.item_state_modifier,
            },
            &vkcl::OverlayOption {
                ckpt: overlay.ckpt,
                ckpt_side: overlay.ckpt_side,
                inv_walls: overlay.inv_walls,
                gobj: overlay.gobj,
            },
            write_obj,
        )
        .map_err(|e| PyRuntimeError::new_err(e))
    }

    #[pyfunction]
    #[pyo3(signature = (input_path, output_path, overlay, write_obj=false))]
    fn overlay(
        input_path: &str,
        output_path: &str,
        overlay: &OverlayOption,
        write_obj: bool,
    ) -> PyResult<()> {
        vkcl::overlay(
            input_path,
            output_path,
            &vkcl::OverlayOption {
                ckpt: overlay.ckpt,
                ckpt_side: overlay.ckpt_side,
                inv_walls: overlay.inv_walls,
                gobj: overlay.gobj,
            },
            write_obj,
        )
        .map_err(|e| PyRuntimeError::new_err(e))
    }

    #[pyfunction]
    #[pyo3(signature = (path, output_path, kcl_options=None, kmp_options=None))]
    fn draw(
        path: &str,
        output_path: &str,
        kcl_options: Option<&KclDrawOptions>,
        kmp_options: Option<&KmpDrawOptions>,
    ) -> PyResult<()> {
        let kcl_default = KclDrawOptions::default();
        let kmp_default = KmpDrawOptions::default();
        let kcl = kcl_options.unwrap_or(&kcl_default);
        let kmp = kmp_options.unwrap_or(&kmp_default);

        let img = vkcl::draw::to_image(
            path,
            &vkcl::KclDrawOptions {
                wireframe: kcl.wireframe,
                shading: kcl.shading,
            },
            &vkcl::KmpDrawOptions {
                thickness: kmp.thickness,
                ktpt: kmp.ktpt,
                enpt: kmp.enpt,
                itpt: kmp.itpt,
                ckpt: kmp.ckpt,
                ckpt_side_lines: kmp.ckpt_side_lines,
                gobj: kmp.gobj,
                poti: kmp.poti,
                area: kmp.area,
                came: kmp.came,
                jgpt: kmp.jgpt,
                jgpt_lines: kmp.jgpt_lines,
                cnpt: kmp.cnpt,
                mspt: kmp.mspt,
                stgi: kmp.stgi,
            },
        );
        img.save(output_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}
