extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;
extern crate time;

use piston::window::WindowSettings;
use piston::event::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input;
use rand::distributions::{IndependentSample, Range};

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64 ,
    snake_width:f64,
    snake_body:Vec<(f64,f64)>,
    direction:&'static str,
    last_auto_move_time:time::Tm,
    food_location:(f64,f64),
    window_dimension:u32,
    previous_tail:(f64,f64)
}

fn generate_random_ordered_pair(grid_length:f64)->(f64,f64){
    println!("{:?}",grid_length);
    let between = Range::new(0, grid_length as i64);
    let mut rng = rand::thread_rng();
    let mut sum = 0;
    let mut tuple=(0.0,0.0);
    tuple.0=(between.ind_sample(&mut rng) as f64)/grid_length;
    tuple.1=(between.ind_sample(&mut rng) as f64)/grid_length;
    println!("{:?}",tuple);
    tuple
}

fn move_up_all_but_first(old_vec:&Vec<(f64,f64)>,new_vec:&mut Vec<(f64,f64)>)->(){
    for (i,tuple) in old_vec.iter().enumerate(){
        if i==0{
            new_vec.push((0.0,0.0))
        }else{
            new_vec.push(old_vec[i-1].clone());
        }
    }
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.auto_move();
        self.eat_food();
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, self.snake_width);
        let food_ellipse = rectangle::square(0.0, 0.0, self.snake_width);
        let rotation = self.rotation;
        let snake_body=&self.snake_body;
        let food_location=self.food_location;
        let window_dimension=self.window_dimension;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
            for tuple in snake_body.iter(){
                let transform = c.transform.rot_rad(rotation).trans(tuple.0,tuple.1);
                // Draw a box rotating around the middle of the screen.
                rectangle(RED, square, transform, gl);
            }
            let food_transform=c.transform.trans(food_location.0*(window_dimension as f64),food_location.1*(window_dimension as f64));
            ellipse(RED, food_ellipse, food_transform, gl);

        });
    }


    fn move_square(&mut self,press_args:input::Button)->(){
        self.previous_tail=self.snake_body[self.snake_body.len()-1];
        let snake_width=self.snake_width;
        let mut new_snake_body=vec![];
        move_up_all_but_first(&self.snake_body,&mut new_snake_body);
        match press_args{
            input::Button::Keyboard(key)=>if key==input::keyboard::Key::Right{
                    if !(self.direction=="left"){
                        new_snake_body[0].0=self.snake_body[0].0+snake_width;
                        new_snake_body[0].1=self.snake_body[0].1;
                        self.direction="right";
                    }else{
                        return ()
                    }
                }else if key==input::keyboard::Key::Left{
                    if !(self.direction=="right"){
                        new_snake_body[0].0=self.snake_body[0].0-snake_width;
                        new_snake_body[0].1=self.snake_body[0].1;
                        self.direction="left";
                    }else{
                        return ()
                    }
                    }else if key==input::keyboard::Key::Up{
                        if !(self.direction=="down"){
                            new_snake_body[0].1=self.snake_body[0].1-snake_width;
                            new_snake_body[0].0=self.snake_body[0].0;
                            self.direction="up";
                        }else{
                            return ()
                        }
                        }else if key==input::keyboard::Key::Down{
                            if !(self.direction=="up"){
                                new_snake_body[0].1=self.snake_body[0].1+snake_width;
                                new_snake_body[0].0=self.snake_body[0].0;
                                self.direction="down";
                            }else{
                                return ()
                            }
                            },
            input::Button::Mouse(key)=>println!("{:?}",key)
        }
        self.last_auto_move_time=time::now();
        self.snake_body=new_snake_body;
    }

    fn game_over(&mut self,r:RenderArgs)->bool{
        let mut game_over=false;
        for tuple in self.snake_body.iter(){
            if tuple.0+self.snake_width>r.width as f64||tuple.0<0.0||tuple.1+self.snake_width>r.height as f64||tuple.1<0.0{
                return true
            }
        }
        game_over
    }

    fn auto_move(&mut self)->(){
        let elapsed=time::now()-self.last_auto_move_time;
        let snake_width=self.snake_width;
        if elapsed>time::Duration::seconds(1){
            println!("{:?}","Automove");
            self.previous_tail=self.snake_body[self.snake_body.len()-1];
            self.last_auto_move_time=time::now();
            let mut new_snake_body=vec![];
            move_up_all_but_first(&self.snake_body,&mut new_snake_body);
            let new_tuple = match self.direction{
                    "down"=>(self.snake_body[0].0,self.snake_body[0].1+snake_width),
                    "up"=>(self.snake_body[0].0,self.snake_body[0].1-snake_width),
                    "left"=>(self.snake_body[0].0-snake_width,self.snake_body[0].1),
                    "right"=>(self.snake_body[0].0+snake_width,self.snake_body[0].1),
                    _=>(0.0,0.0)
                };
            new_snake_body[0]=new_tuple;
            self.snake_body=new_snake_body;
            println!("{:?}",self.snake_body);
        }

    }

    fn eat_food(&mut self)->(){
        let food_x=(self.food_location.0*(self.window_dimension as f64)) as i64;
        let food_y=(self.food_location.1*(self.window_dimension as f64)) as i64;
        let snake_x=(self.snake_body[0].0) as i64;
        let snake_y=(self.snake_body[0].1) as i64;
        if (food_x,food_y)==(snake_x,snake_y){
            self.food_location=generate_random_ordered_pair((self.window_dimension as f64)/self.snake_width);
            match self.direction{
                "up"=>self.snake_body.push(self.previous_tail),
                "down"=>self.snake_body.push(self.previous_tail),
                "left"=>self.snake_body.push(self.previous_tail),
                "right"=>self.snake_body.push(self.previous_tail),
                _=>self.snake_body.push(self.previous_tail)
            }
        }
        //println!("{:?}",(self.food_location.0*(self.window_dimension as f64),self.food_location.1*((self.window_dimension as f64))));
        //println!("{:?}",self.snake_body[0]);
    }
}


//RenderArgs { ext_dt: 0.000002581, width: 200, height: 200, draw_width: 200, draw_height: 200 }
fn main() {
    let opengl = OpenGL::_3_2;
    let window_dimension=200;
    let snake_width=10.0;
    // Create an Glutin window.
    let window = Window::new(
        WindowSettings::new(
            "Snake",
            [window_dimension, window_dimension]
        )
        .exit_on_esc(true)
        .opengl(opengl)
    );

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        window_dimension:window_dimension,
        snake_width:snake_width,
        rotation: 0.0,
        snake_body:vec![(0.0,0.0)],
        direction:"down",
        last_auto_move_time:time::now(),
        food_location:generate_random_ordered_pair((window_dimension as f64)/snake_width),
        previous_tail:(0.0,0.0)
    };
    println!("{:?}",time::now());
    for e in window.events() {
        match e.press_args(){
            Some(press_args)=>app.move_square(press_args),
            None=>()
        }
        //println!("{:?}",e.render_args());
        //println!("{:?}",e.update_args());
        if let Some(r) = e.render_args() {
            if app.game_over(r){
                println!("{:?}","You Lose!");
                println!("{:?}",r);
                break;
            }
          app.render(&r);
        }

        //if let Some(u) = e.update_args() {
        //    app.update(&u);
        //}
    }


}
