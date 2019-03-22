use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{run, EventHandler},
    graphics::{draw, present},
    input::keyboard::{KeyCode, KeyMods},
    mint::Point2 as MintPoint2,
    nalgebra::Point2,
    quit,
    timer::yield_now,
    Context,
    ContextBuilder,
    GameResult,
};
use rand::{rngs::ThreadRng, thread_rng};
use snake::{
    clear_screen,
    snake_state::{state as get_snake_state, SnakeState},
    validate_next_direction,
    CoordT,
    Direction,
    Food,
    MeshRepr,
    Snake,
    DEFAULT_DIRECTION,
    DEFAULT_SNAKE_COORD,
    GRID_DIMENSIONS,
};
use std::time::{Duration, Instant};

mod snake;

const UPDATE_MILLIS: u64 = 110;

fn main() -> GameResult {
    let (mut context, mut event_loop) =
        ContextBuilder::new("Snake Game", "Amrit Rathie")
            .window_setup(WindowSetup::default().title("Snake!"))
            .window_mode(WindowMode::default().resizable(true))
            .build()?;

    let mut state = GameState::new(DEFAULT_SNAKE_COORD);

    run(&mut context, &mut event_loop, &mut state)
}

struct GameState {
    snake: Snake,
    food: Food,
    snake_state: SnakeState,
    input_direction: Direction,
    game_over: bool,
    last_update: Instant,
    rng: ThreadRng,
}

impl GameState {
    pub fn new<S: Into<Point2<CoordT>>>(snake_coord: S) -> Self {
        let mut rng = thread_rng();
        let snake = Snake::new(snake_coord);
        let food = Food::random(&mut rng, GRID_DIMENSIONS.0, GRID_DIMENSIONS.1);
        let snake_state = get_snake_state(&snake, &food);
        GameState {
            snake,
            food,
            snake_state,
            rng,
            input_direction: DEFAULT_DIRECTION,
            game_over: false,
            last_update: Instant::now(),
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if Instant::now() - self.last_update
            >= Duration::from_millis(UPDATE_MILLIS)
        {
            if !self.game_over {
                match self.snake_state {
                    SnakeState::Food => {
                        let direction = self.input_direction;
                        let new_head = self
                            .snake
                            .compute_next_head(direction)
                            .expect(
                            "Tried to compute the next head of an empty snake",
                        );
                        self.snake.compute_next_frame_with_new(new_head);
                        self.food = Food::random(
                            &mut self.rng,
                            GRID_DIMENSIONS.0,
                            GRID_DIMENSIONS.1,
                        );
                        // TODO: make sure food does not spawn on body
                        self.snake_state =
                            get_snake_state(&self.snake, &self.food);
                    },
                    SnakeState::Segment => {
                        self.game_over = true;
                        // BUG: might need to check state first instead of after
                        // so that game ends within the correct frame
                    },
                    SnakeState::Nothing => {
                        let direction = self.input_direction;
                        let new_head = self
                            .snake
                            .compute_next_head(direction)
                            .expect(
                            "Tried to  compute the next head of an empty snake",
                        );
                        self.snake.compute_next_frame(new_head);
                        self.snake_state =
                            get_snake_state(&self.snake, &self.food);
                    },
                }
            } else {
                // Have a game over state, don't just quit
                quit(ctx);
            }
            self.last_update = Instant::now();
        } else {
            // COMBAK: test with and without this
            yield_now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        clear_screen(ctx);
        let snake_mesh = self.snake.get_mesh(ctx)?;
        draw(
            ctx,
            &snake_mesh,
            (MintPoint2 {
                x: 0.0,
                y: 0.0,
            },),
        )?;
        let food_mesh = self.food.get_mesh(ctx)?;
        draw(
            ctx,
            &food_mesh,
            (MintPoint2 {
                x: 0.0,
                y: 0.0,
            },),
        )?;
        present(ctx)?;
        yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if let Some(direction) = match keycode {
            KeyCode::Up | KeyCode::W => Some(Direction::Up),
            KeyCode::Down | KeyCode::S => Some(Direction::Down),
            KeyCode::Left | KeyCode::A => Some(Direction::Left),
            KeyCode::Right | KeyCode::D => Some(Direction::Right),
            _ => None,
        } {
            if validate_next_direction(direction, self.input_direction) {
                self.input_direction = direction;
            }
        }
    }
}
