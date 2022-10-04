//! This example shows how to use touch events in `Canvas` to draw
//! a circle around each fingertip. This only works on touch-enabled
//! computers like Microsoft Surface.
use iced::widget::canvas::event;
use iced::widget::canvas::{self, Canvas, Cursor, Geometry, Stroke};
use iced::{
    executor, touch, window, Application, Color, Command, Element, Length,
    Point, Rectangle, Settings, Subscription, Theme,
};

use std::collections::HashMap;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    Multitouch::run(Settings {
        antialiasing: true,
        window: window::Settings {
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct Multitouch {
    state: State,
}

#[derive(Debug)]
struct State {
    cache: canvas::Cache,
    fingers: HashMap<touch::Finger, Point>,
}

impl State {
    fn new() -> Self {
        Self {
            cache: canvas::Cache::new(),
            fingers: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum Message {
    FingerPressed { id: touch::Finger, position: Point },
    FingerLifted { id: touch::Finger },
}

impl Application for Multitouch {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Multitouch {
                state: State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Multitouch - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FingerPressed { id, position } => {
                self.state.fingers.insert(id, position.clone());
                self.state.cache.clear();
            }
            Message::FingerLifted { id } => {
                self.state.fingers.remove(&id);
                self.state.cache.clear();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(&self.state)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<'a> canvas::Program<Message> for State {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: event::Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        match event {
            event::Event::Touch(touch_event) => match touch_event {
                touch::Event::FingerPressed { id, position }
                | touch::Event::FingerMoved { id, position } => (
                    event::Status::Captured,
                    Some(Message::FingerPressed { id, position }),
                ),
                touch::Event::FingerLifted { id, .. }
                | touch::Event::FingerLost { id, .. } => (
                    event::Status::Captured,
                    Some(Message::FingerLifted { id }),
                ),
            },
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let fingerweb = self.cache.draw(bounds.size(), |frame| {
            if self.fingers.len() < 2 {
                return;
            }

            // Collect tuples of (id, point);
            let mut zones: Vec<(u64, Point)> = self
                .fingers
                .iter()
                .map(|(id, pt)| (id.0, pt.clone()))
                .collect();

            // Sort by ID
            zones.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // Generate sorted list of points
            let vpoints: Vec<voronoi::Point> =
                zones.iter().map(|zone| to_voronoi_point(&zone.1)).collect();

            let diagram = voronoi::voronoi(vpoints, 700.0);
            let polys = voronoi::make_polygons(&diagram);

            for (poly, zone) in polys.iter().zip(zones) {
                let mut builder = canvas::path::Builder::new();

                for (index, pt) in poly.iter().enumerate() {
                    let pt = from_voronoi_point(pt);

                    match index {
                        0 => builder.move_to(pt),
                        _ => builder.line_to(pt),
                    }
                }

                let path = builder.build();

                let color_r = (10 % zone.0) as f32 / 20.0;
                let color_g = (10 % (zone.0 + 8)) as f32 / 20.0;
                let color_b = (10 % (zone.0 + 3)) as f32 / 20.0;

                frame.fill(
                    &path,
                    Color {
                        r: color_r,
                        g: color_g,
                        b: color_b,
                        a: 1.0,
                    },
                );

                frame.stroke(
                    &path,
                    Stroke {
                        color: Color::BLACK,
                        width: 3.0,
                        ..Stroke::default()
                    },
                );
            }
        });

        vec![fingerweb]
    }
}

fn to_voronoi_point(pt: &iced::Point) -> voronoi::Point {
    voronoi::Point::new(pt.x.into(), pt.y.into())
}

fn from_voronoi_point(pt: &voronoi::Point) -> iced::Point {
    let x: f64 = pt.x.into();
    let y: f64 = pt.y.into();

    Point {
        x: x as f32,
        y: y as f32,
    }
}
