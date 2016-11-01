extern crate cairo;

#[repr(C)]
pub struct RsvgPathBuilder {
    path_segments: Vec<cairo::PathSegment>,
    last_move_to_index: Option<usize>
}

impl RsvgPathBuilder {
    fn move_to (&mut self, x: f64, y: f64) {
        self.path_segments.push (cairo::PathSegment::MoveTo ((x, y)));
        self.last_move_to_index = Some (self.path_segments.len () - 1);
    }

    fn line_to (&mut self, x: f64, y: f64) {
        self.path_segments.push (cairo::PathSegment::LineTo ((x, y)));
    }

    fn curve_to (&mut self, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64) {
        self.path_segments.push (cairo::PathSegment::CurveTo ((x2, y2), (x3, y3), (x4, y4)));
    }

    fn close_path (&mut self) {
        if let Some (idx) = self.last_move_to_index {
            let segment = self.path_segments[idx];

            if let cairo::PathSegment::MoveTo ((x, y)) = segment {
                self.move_to (x, y);
            } else {
                unreachable! ();
            }
        }
    }
}

#[no_mangle]
pub unsafe extern fn rsvg_path_builder_new () -> *mut RsvgPathBuilder {
    let builder = RsvgPathBuilder {
        path_segments: Vec::new (),
        last_move_to_index: None
    };

    let boxed_builder = Box::new (builder);

    Box::into_raw (boxed_builder)
}

#[no_mangle]
pub unsafe extern fn rsvg_path_builder_destroy (raw_builder: *mut RsvgPathBuilder) {
    assert! (!raw_builder.is_null ());

    let _ = Box::from_raw (raw_builder);
}

#[no_mangle]
pub extern fn rsvg_path_builder_move_to (raw_builder: *mut RsvgPathBuilder,
                                         x: f64,
                                         y: f64) {
    assert! (!raw_builder.is_null ());

    let builder: &mut RsvgPathBuilder = unsafe { &mut (*raw_builder) };

    builder.move_to (x, y);
}

#[no_mangle]
pub extern fn rsvg_path_builder_line_to (raw_builder: *mut RsvgPathBuilder,
                                         x: f64,
                                         y: f64) {
    assert! (!raw_builder.is_null ());

    let builder: &mut RsvgPathBuilder = unsafe { &mut (*raw_builder) };

    builder.line_to (x, y);
}

#[no_mangle]
pub extern fn rsvg_path_builder_curve_to (raw_builder: *mut RsvgPathBuilder,
                                          x2: f64, y2: f64,
                                          x3: f64, y3: f64,
                                          x4: f64, y4: f64) {
    assert! (!raw_builder.is_null ());

    let builder: &mut RsvgPathBuilder = unsafe { &mut (*raw_builder) };

    builder.curve_to (x2, y2, x3, y3, x4, y4);
}

#[no_mangle]
pub extern fn rsvg_path_builder_close_path (raw_builder: *mut RsvgPathBuilder) {
    assert! (!raw_builder.is_null ());

    let builder: &mut RsvgPathBuilder = unsafe { &mut (*raw_builder) };

    builder.close_path ();
}
