use crate::api::{CameraController, CameraMode};
use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{
            MouseMotion,
            MouseScrollUnit::{Line, Pixel},
            MouseWheel,
        },
    },
    prelude::*,
    render::camera::Camera,
};
use bevy_mod_picking::{
    debug::DebugPickingPlugin,
    pointer::{self, InputMove, InputPress},
    prelude::{Click, Down, Drag, DragEnd, DragStart, Pointer, PointerButton, Up},
    DefaultPickingPlugins,
};
use std::ops::RangeInclusive;

#[derive(Default)]
pub struct OrbitCameraControllerPlugin<T: CameraMode>(pub T);

impl<T: CameraMode + Send + Sync + 'static> Plugin for OrbitCameraControllerPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(First, Self::init_camera_state)
            .add_event::<OrbitCameraControllerEvents>()
            .add_systems(
                PostUpdate,
                (Self::emit_motion_events).run_if(
                    not(on_event::<Pointer<DragStart>>())
                        .and_then(not(on_event::<Pointer<Drag>>()))
                        .and_then(not(on_event::<Pointer<DragEnd>>()))
                        .and_then(not(on_event::<Pointer<Down>>()))
                        .and_then(not(on_event::<Pointer<Up>>()))
                        .and_then(not(on_event::<Pointer<Click>>()))
                        .and_then(run_criteria::<T>),
                ),
            )
            .add_systems(Update, (Self::emit_zoom_events, Self::emit_keyboard_events))
            .add_systems(
                Last,
                (
                    Self::consume_pan_and_orbit_events,
                    Self::consume_zoom_events,
                    Self::consume_recentre_events,
                    Self::update_camera_transform_system,
                )
                    .chain()
                    .run_if(on_event::<OrbitCameraControllerEvents>()),
            );
    }
}

const LINE_TO_PIXEL_RATIO: f32 = 0.1;

fn run_criteria<T: CameraMode>(mode: Res<T>) -> bool {
    !(*mode).is_locked()
}

#[derive(Event)]
pub enum OrbitCameraControllerEvents {
    Orbit(Vec2),
    Pan(Vec2),
    Zoom(f32),
    Recentre(Vec3),
}

#[derive(Component)]
pub struct OrbitCameraController {
    pub θ: f32,
    pub ψ: f32,
    pub ρ: f32,
    pub θ_range: RangeInclusive<f32>,
    pub ψ_range: RangeInclusive<f32>,
    pub center: Vec3,
    pub rotate_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub change_centre_sensitivity: f32,
    pub rotate_button: MouseButton,
    pub pan_button: MouseButton,
    pub enabled: bool,
}

// There are many conventions on ψ and θ: we'll take the default one from Riley, Hobson, Bence
impl Default for OrbitCameraController {
    fn default() -> Self {
        OrbitCameraController {
            // The initial rotation around the Y axis (in radians). 0.0 means facing directly towards the north or forward direction.
            θ: 0.0,
            ψ: std::f32::consts::FRAC_PI_2,
            θ_range: 0.01..=std::f32::consts::PI,
            ψ_range: 0.01..=2. * std::f32::consts::PI,
            ρ: 100.,
            // The point in world space that the camera orbits around. Vec3::ZERO is the origin (0, 0, 0).
            center: Vec3::ZERO,
            // Sensitivity of the camera rotation. Lower values make the camera rotate slower.
            rotate_sensitivity: 0.4,
            // Sensitivity of the camera panning. Lower values make the camera pan slower.
            pan_sensitivity: 0.4,
            // Sensitivity of the camera zooming (changing the ρ). Lower values zoom slower.
            zoom_sensitivity: 0.4,
            // change centrepoint sensitivity
            change_centre_sensitivity: 0.4,
            // Mouse button used to rotate the camera. Typically, the left mouse button.
            rotate_button: MouseButton::Left,
            // Mouse button used to pan the camera. Typically, the right mouse button.
            pan_button: MouseButton::Right,
            // Whether the camera controller is enabled. If false, the camera won't respond to input.
            enabled: true,
        }
    }
}

impl OrbitCameraController {
    pub fn new(dist: f32, center: Vec3) -> OrbitCameraController {
        OrbitCameraController {
            ρ: dist,
            center,
            ..Self::default()
        }
    }
    pub fn ρ_basis_vector(&self) -> Vec3 {
        let &Self { θ, ρ, ψ, .. } = self;

        let x = ρ * f32::cos(ψ) * f32::cos(θ);
        let y = ρ * f32::sin(ψ) * f32::sin(θ);
        let z = ρ * f32::cos(θ);

        Vec3::new(x, y, z).normalize()
    }
    pub fn ψ_basis_vector(&self) -> Vec3 {
        let &Self { θ, ρ, ψ, .. } = self;

        let x: f32 = -f32::sin(ψ);
        let y: f32 = -f32::cos(ψ);
        let z = 0.;

        Vec3::new(x, y, z).normalize()
    }
    pub fn θ_basis_vector(&self) -> Vec3 {
        let &Self { θ, ρ, ψ, .. } = self;

        let x: f32 = f32::cos(ψ) * f32::cos(θ);
        let y = f32::sin(ψ) * f32::cos(θ);
        let z = -f32::sin(θ);

        Vec3::new(x, y, z).normalize()
    }
}

impl CameraController for OrbitCameraController {
    fn update_camera_transform_system(
        mut query: Query<
            (&OrbitCameraController, &mut Transform),
            (Changed<OrbitCameraController>, With<Camera>),
        >,
    ) {
        for (camera, mut transform) in query.iter_mut() {
            if camera.enabled {
                let rot = Quat::from_axis_angle(Vec3::Y, camera.ψ)
                    * Quat::from_axis_angle(-Vec3::X, camera.θ);
                transform.translation = (rot * Vec3::Y) * camera.ρ + camera.center;
                transform.look_at(camera.center, Vec3::Y);
            }
        }
    }
}

impl<T: CameraMode> OrbitCameraControllerPlugin<T> {
    pub fn init_camera_state(mut commands: Commands) {
        commands.init_resource::<T>()
    }

    pub fn update_camera_transform_system(
        query: Query<
            (&OrbitCameraController, &mut Transform),
            (Changed<OrbitCameraController>, With<Camera>),
        >,
    ) {
        OrbitCameraController::update_camera_transform_system(query);
    }

    pub fn emit_motion_events(
        mut events: EventWriter<OrbitCameraControllerEvents>,
        mut pointer_motion_events: EventReader<InputMove>,
        pointer_button_input: Res<ButtonInput<MouseButton>>,
        mut query: Query<&OrbitCameraController>,
    ) {
        let mut delta = Vec2::ZERO;
        for event in pointer_motion_events.read() {
            delta += event.delta;
        }
        for camera in query.iter_mut() {
            if camera.enabled {
                if pointer_button_input.pressed(camera.rotate_button) {
                    events.send(OrbitCameraControllerEvents::Orbit(delta));
                }

                if pointer_button_input.pressed(camera.pan_button) {
                    events.send(OrbitCameraControllerEvents::Pan(delta));
                }
            }
        }
    }

    pub fn emit_keyboard_events(
        // Input
        mut keyboard_presses: EventReader<KeyboardInput>,
        // Output
        mut camera_cmd_events: EventWriter<OrbitCameraControllerEvents>,
        mut query: Query<&OrbitCameraController>,
    ) {
        for camera in query.iter_mut() {
            if camera.enabled {
                for kbd in keyboard_presses.read() {
                    match kbd.key_code {
                        KeyCode::ArrowDown | KeyCode::KeyS => {
                            camera_cmd_events.send(OrbitCameraControllerEvents::Recentre(
                                -camera.change_centre_sensitivity * camera.ψ_basis_vector(),
                            ));
                        }
                        KeyCode::ArrowUp | KeyCode::KeyW => {
                            camera_cmd_events.send(OrbitCameraControllerEvents::Recentre(
                                camera.change_centre_sensitivity * camera.ψ_basis_vector(),
                            ));
                        }
                        KeyCode::ArrowLeft | KeyCode::KeyA => {
                            camera_cmd_events.send(OrbitCameraControllerEvents::Recentre(
                                -camera.change_centre_sensitivity * camera.θ_basis_vector(),
                            ));
                        }
                        KeyCode::ArrowRight | KeyCode::KeyD => {
                            camera_cmd_events.send(OrbitCameraControllerEvents::Recentre(
                                camera.change_centre_sensitivity * camera.θ_basis_vector(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn consume_pan_and_orbit_events(
        time: Res<Time>,
        mut events: EventReader<OrbitCameraControllerEvents>,
        mut query: Query<(&mut OrbitCameraController, &mut Transform, &mut Camera)>,
    ) {
        for (mut camera, transform, _) in query.iter_mut() {
            if !camera.enabled {
                continue;
            }

            for event in events.read() {
                match event {
                    OrbitCameraControllerEvents::Orbit(delta) => {
                        camera.ψ -= delta.x * camera.rotate_sensitivity * time.delta_seconds();
                        camera.θ -= delta.y * camera.rotate_sensitivity * time.delta_seconds();
                        camera.θ = camera
                            .θ
                            .max(*camera.θ_range.start())
                            .min(*camera.θ_range.end());
                    }
                    OrbitCameraControllerEvents::Pan(delta) => {
                        let right_dir = transform.rotation * -Vec3::X;
                        let up_dir = transform.rotation * Vec3::Y;
                        let pan_vector = (delta.x * right_dir + delta.y * up_dir)
                            * camera.pan_sensitivity
                            * time.delta_seconds();
                        camera.center += pan_vector;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn consume_recentre_events(
        mut events: EventReader<OrbitCameraControllerEvents>,
        mut query: Query<(&mut OrbitCameraController, &mut Transform, &mut Camera)>,
    ) {
        for (mut camera, transform, _) in query.iter_mut() {
            if !camera.enabled {
                continue;
            }

            for event in events.read() {
                match event {
                    OrbitCameraControllerEvents::Recentre(dr) => {
                        camera.center += *dr;
                    }

                    _ => {}
                }
            }
        }
    }

    pub fn emit_zoom_events(
        mut events: EventWriter<OrbitCameraControllerEvents>,
        mut mouse_wheel_events: EventReader<MouseWheel>,
        mut query: Query<&OrbitCameraController>,
    ) {
        let mut total = 0.0;
        for event in mouse_wheel_events.read() {
            total += event.y
                * match event.unit {
                    Line => 1.0,
                    Pixel => LINE_TO_PIXEL_RATIO,
                };
        }

        if total != 0.0 {
            for camera in query.iter_mut() {
                if camera.enabled {
                    events.send(OrbitCameraControllerEvents::Zoom(total));
                }
            }
        }
    }

    pub fn consume_zoom_events(
        mut query: Query<&mut OrbitCameraController, With<Camera>>,
        mut events: EventReader<OrbitCameraControllerEvents>,
    ) {
        for mut camera in query.iter_mut() {
            for event in events.read() {
                if camera.enabled {
                    if let OrbitCameraControllerEvents::Zoom(ρ) = event {
                        camera.ρ += camera.zoom_sensitivity * (*ρ);
                        camera.ρ = f32::clamp(camera.ρ, 0.1, f32::MAX)
                    }
                }
            }
        }
    }
}
