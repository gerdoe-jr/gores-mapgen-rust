use vek::Vec2;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, ElementState, MouseButton, Touch, TouchPhase};

use twgpu::Camera;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum MultiInput {
    #[default]
    None,
    One(InputDevice),
    Two {
        one: InputDevice,
        two: InputDevice,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct InputDevice {
    id: Id,
    logical_pos: Vec2<f32>,
    map_pos: Vec2<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Input {
    kind: InputKind,
    id: Id,
    action: Action,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum InputKind {
    Cursor,
    Touch,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Id {
    Device(DeviceId),
    Num(u64),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Action {
    Position(Vec2<f32>),
    Remove,
}

fn phys_pos<T>(phys: PhysicalPosition<T>) -> Vec2<T> {
    Vec2::new(phys.x, phys.y)
}

impl Input {
    pub fn from_touch(touch: Touch) -> Self {
        Self {
            kind: InputKind::Touch,
            id: Id::Num(touch.id),
            action: match touch.phase {
                TouchPhase::Started | TouchPhase::Moved => {
                    Action::Position(Vec2::new(touch.location.x, touch.location.y).az())
                }
                TouchPhase::Ended | TouchPhase::Cancelled => Action::Remove,
            },
        }
    }
}

impl InputDevice {
    fn new(input: &Input, camera: &Camera, render_size: Vec2<f32>) -> Self {
        let logical_pos = match input.action {
            Action::Position(pos) => pos / render_size,
            Action::Remove => panic!(),
        };
        InputDevice {
            id: input.id,
            logical_pos,
            map_pos: camera.map_position(logical_pos),
        }
    }

    fn update(&mut self, input: &Input, render_size: Vec2<f32>) -> bool {
        if self.id == input.id {
            let screen_pos = match input.action {
                Action::Position(pos) => pos,
                Action::Remove => panic!(),
            };
            self.logical_pos = screen_pos / render_size;
            true
        } else {
            false
        }
    }

    fn update_map_pos(&mut self, camera: &Camera) {
        self.map_pos = camera.map_position(self.logical_pos);
    }
}

impl MultiInput {
    /// Backup is only used in case of no active `InputDevices.
    /// It is the physical position that acts as the anchor point.
    /// The map-coordinates of that position will stay the same independant of zoom.
    pub fn update_camera(
        &self,
        camera: &mut Camera,
        old_camera: &Camera,
        render_size: Vec2<f32>,
        backup: Option<PhysicalPosition<f64>>,
    ) {
        match self {
            MultiInput::None => {
                if let Some(backup) = backup {
                    let logical_pos = phys_pos(backup).az() / render_size;
                    let before_map_pos = old_camera.map_position(logical_pos);
                    camera.move_to(before_map_pos, logical_pos);
                }
            }
            MultiInput::One(only) => camera.move_to(only.map_pos, only.logical_pos),
            MultiInput::Two { one, .. } => camera.move_to(one.map_pos, one.logical_pos),
        }
    }

    pub fn update_map_positions(&mut self, camera: &Camera) {
        match self {
            MultiInput::None => {}
            MultiInput::One(only) => only.update_map_pos(camera),
            MultiInput::Two { one, two, .. } => {
                one.update_map_pos(camera);
                two.update_map_pos(camera);
            }
        }
    }

    /// Returns `true`, if a new device was added
    pub fn update_input(&mut self, input: &Input, camera: &mut Camera, render_size: Vec2<f32>) {
        match input.action {
            Action::Position(_) => {
                let updated = self.update_existing(input, camera, render_size);
                if !updated {
                    let new_device = InputDevice::new(input, camera, render_size);
                    self.add(new_device);
                }
            }
            Action::Remove => self.remove(input.id, camera),
        }
    }

    /// Tries to update a contained `InputDevice`.
    /// Returns `false`, if it doesn't contain a matching `InputDevice`.
    fn update_existing(
        &mut self,
        input: &Input,
        camera: &mut Camera,
        render_size: Vec2<f32>,
    ) -> bool {
        match self {
            MultiInput::None => false,
            MultiInput::One(dev) => dev.update(input, render_size),
            MultiInput::Two { one, two } => {
                let old_distance = one.logical_pos.distance(two.logical_pos);
                let changed = one.update(input, render_size) || two.update(input, render_size);
                if changed {
                    let new_distance = one.logical_pos.distance(two.logical_pos);
                    camera.zoom *= old_distance / new_distance;
                }
                changed
            }
        }
    }

    fn remove(&mut self, id: Id, camera: &Camera) {
        match self {
            MultiInput::None => {}
            MultiInput::One(device) => {
                if device.id == id {
                    *self = MultiInput::None;
                }
            }
            MultiInput::Two { one, two } => {
                if one.id == id {
                    two.update_map_pos(camera);
                    *self = MultiInput::One(*two);
                } else if two.id == id {
                    *self = MultiInput::One(*one);
                }
            }
        }
    }

    fn add(&mut self, device: InputDevice) {
        match self {
            MultiInput::None => *self = MultiInput::One(device),
            MultiInput::One(one) => {
                *self = MultiInput::Two {
                    one: *one,
                    two: device,
                }
            }
            MultiInput::Two { .. } => {}
        }
    }
}

struct Cursor {
    id: DeviceId,
    active: bool,
    position: Option<PhysicalPosition<f64>>,
}

#[derive(Default)]
pub struct Cursors(Vec<Cursor>);

impl Cursors {
    fn cursor_index(&self, id: DeviceId) -> Option<usize> {
        self.0.iter().position(|c| c.id == id)
    }

    fn cursor(&mut self, id: DeviceId) -> Option<&mut Cursor> {
        let index = self.cursor_index(id)?;
        Some(&mut self.0[index])
    }

    /// Queuries for any cursor position
    pub fn any_position(&self) -> Option<PhysicalPosition<f64>> {
        self.0.iter().find(|c| c.position.is_some())?.position
    }

    pub fn entered(&mut self, id: DeviceId) {
        self.0.push(Cursor {
            id,
            active: false,
            position: None,
        })
    }

    pub fn left(&mut self, id: DeviceId) {
        if let Some(index) = self.cursor_index(id) {
            self.0.remove(index);
        }
    }

    pub fn moved(&mut self, id: DeviceId, position: PhysicalPosition<f64>) -> Option<Input> {
        let cursor = self.cursor(id)?;
        cursor.position = Some(position);
        if cursor.active {
            Some(Input {
                kind: InputKind::Cursor,
                id: Id::Device(cursor.id),
                action: Action::Position(phys_pos(position).az()),
            })
        } else {
            None
        }
    }

    pub fn input(
        &mut self,
        id: DeviceId,
        state: ElementState,
        button: MouseButton,
    ) -> Option<Input> {
        if button != MouseButton::Left {
            return None;
        }
        let cursor = self.cursor(id)?;
        let state = match state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };
        cursor.active = state;
        Some(Input {
            kind: InputKind::Cursor,
            id: Id::Device(id),
            action: match state {
                true => Action::Position(phys_pos(cursor.position?).az()),
                false => Action::Remove,
            },
        })
    }
}
