use makepad_widgets::*;
use std::path::{Path, PathBuf};

live_design! {
    use link::widgets::*;

    LEFT_ARROW = dep("crate://self/resources/left_arrow.svg");
    RIGHT_ARROW = dep("crate://self/resources/right_arrow.svg");
    PLACEHOLDER = dep("crate://self/resources/placeholder.png");

    MenuBarButton = <Button> {
        text: "Slideshow",
    }

    MenuBar = <View> {
        width: Fill,
        height: Fit,
        align: {
            x: 1.0,
        },

        button = <MenuBarButton> {}
    }

    ImageGridItem = <View> {
        width: 256,
        height: 256,

        image = <Image> {
            width: Fill,
            height: Fill,
            fit: Biggest,
            source: (PLACEHOLDER),
        }
    }

    ImageGridRow = {{ImageGridRow}} {
        items = <PortalList> {
            height: 256,
            flow: Right,

            scroll_bar: {
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        return vec4(0.0, 0.0, 0.0, 0.0);
                    }
                },
            },

            Item = <ImageGridItem> {}
        }
    }

    ImageGrid = {{ImageGrid}} {
        rows = <PortalList> {
            flow: Down,

            scroll_bar: {
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        return vec4(0.0, 0.0, 0.0, 0.0);
                    }
                },
            },
            
            Row = <ImageGridRow> {}
        }
    }

    ImageBrowser = <View> {
        flow: Down,

        menu_bar = <MenuBar> {}
        image_grid = <ImageGrid> {}
    }

    SlideshowButton = <Button> {
        margin: 0,
        width: 50,
        height: Fill,
        draw_bg: {
            color: #FFF0,
            color_down: #FFF2,
        },
        icon_walk: {
            width: 10
        },
        grab_key_focus: false,
    }

    SlideshowOverlay = <View> {
        cursor: Arrow,
        capture_overload: true,

        left_button = <SlideshowButton> {
            draw_icon: {
                svg_file: (LEFT_ARROW)
            }
        }
        <Filler> {}
        right_button = <SlideshowButton> {
            draw_icon: {
                svg_file: (RIGHT_ARROW)
            }
        }
    }

    Slideshow = <View> {
        flow: Overlay,

        image = <Image> {
            width: Fill,
            height: Fill,
            fit: Biggest,
            source: (PLACEHOLDER)
        }
        overlay = <SlideshowOverlay> {}
    }

    App = {{App}} {
        ui: <Root> {
            <Window> {
                body = <View> {
                    image_browser = <ImageBrowser> {}
                }
            }
        }
        placeholder: (PLACEHOLDER)
    }
}

#[derive(Live)]
struct App {
    #[live]
    ui: WidgetRef,
    #[live]
    placeholder: LiveDependency,
    #[rust]
    state: State,
}

impl App {
    fn load_image_paths(&mut self, cx: &mut Cx, dir: &Path) {
        self.state.image_paths.clear();

        for entry in dir.read_dir().unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                self.state.image_paths.push(path);
            }
        }

        self.set_current_image(cx, 0);
    }

    fn set_current_image(&mut self, cx: &mut Cx, image_idx: usize) {
        self.state.current_image_idx = image_idx;

        let image = self.ui.image(id!(slideshow.image));
        if let Some(path) = self.state.image_paths.get(image_idx) {
            image.load_image_file_by_path_async(cx, &path).unwrap();
        } else {
            let placeholder = self.placeholder.as_str();
            image.load_image_dep_by_path(cx, placeholder).unwrap();
        }

        self.ui.redraw(cx);
    }

    fn go_to_previous_image(&mut self, cx: &mut Cx) {
        if self.state.current_image_idx > 0 {
            self.set_current_image(cx, self.state.current_image_idx - 1);
        }
    }

    fn go_to_next_image(&mut self, cx: &mut Cx) {
        if self.state.current_image_idx + 1 < self.state.num_images() {
            self.set_current_image(cx, self.state.current_image_idx + 1);
        }
    }
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl LiveHook for App {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.load_image_paths(cx, "../../images".as_ref());
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        let mut scope = Scope::with_data(&mut self.state);
        self.ui.handle_event(cx, event, &mut scope);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(id!(left_button)).clicked(&actions) {
            self.go_to_previous_image(cx);
        }
        if self.ui.button(id!(right_button)).clicked(&actions) {
            self.go_to_next_image(cx);
        }

        if let Some(event) = self.ui.view(id!(overlay)).key_down(&actions) {
            match event.key_code {
                KeyCode::ArrowLeft => self.go_to_previous_image(cx),
                KeyCode::ArrowRight => self.go_to_next_image(cx),
                _ => {}
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ImageGrid {
    #[deref]
    view: View,
}

impl Widget for ImageGrid {
    fn draw_walk(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        walk: Walk,
    ) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            let state = scope.data.get_mut::<State>().unwrap();

            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, state.num_rows());

                while let Some(row_idx) = list.next_visible_item(cx) {
                    if row_idx >= state.num_rows() {
                        continue;
                    }

                    let row = list.item(cx, row_idx, live_id!(Row));
                    let mut scope = Scope::with_data_props(state, &row_idx);
                    row.draw_all(cx, &mut scope);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ImageGridRow {
    #[deref]
    view: View,
}

impl Widget for ImageGridRow {
    fn draw_walk(
        &mut self,
        cx: &mut Cx2d,
        scope: &mut Scope,
        walk: Walk,
    ) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            let state = scope.data.get_mut::<State>().unwrap();
            let row_idx = *scope.props.get::<usize>().unwrap();

            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, state.num_images_for_row(row_idx));

                while let Some(item_idx) = list.next_visible_item(cx) {
                    if item_idx >= state.num_images_for_row(row_idx) {
                        continue;
                    }

                    let item = list.item(cx, item_idx, live_id!(Item));

                    let image = item.image(id!(image));
                    let first_image_idx = state.first_image_for_row(row_idx);
                    let image_idx = first_image_idx + item_idx;
                    let image_path = &state.image_paths[image_idx];
                    image.load_image_file_by_path_async(cx, &image_path).unwrap();

                    item.draw_all(cx, &mut Scope::empty());
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope)
    }
}

struct State {
    image_paths: Vec<PathBuf>,
    max_images_per_row: usize,
    current_image_idx: usize,
}

impl State {
    fn num_images(&self) -> usize {
        self.image_paths.len()
    }

    fn num_rows(&self) -> usize {
        self.num_images().div_ceil(self.max_images_per_row)
    }

    fn first_image_for_row(&self, row_idx: usize) -> usize {
        row_idx * self.max_images_per_row
    }

    fn num_images_for_row(&self, row_idx: usize) -> usize {
        let first_image_idx = self.first_image_for_row(row_idx);
        let num_remaining_images = self.num_images() - first_image_idx;
        num_remaining_images.min(self.max_images_per_row)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            image_paths: Vec::new(),
            max_images_per_row: 4,
            current_image_idx: 0,
        }
    }
}

app_main!(App);
