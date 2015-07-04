extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;
extern crate time;
extern crate conrod;

use piston::window::WindowSettings;
use piston::event::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input;
use rand::distributions::{IndependentSample, Range};
use std::path::Path;
use graphics::character::CharacterCache;
use conrod::{Background, Button,CanvasId,DropDownList, Floating, Label, Labelable,Positionable, Sizeable, Theme, Ui,UiId, Widget,WidgetId,Split};
use conrod::color::*;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64 ,
    snake_width:f64,
    snake_body:Vec<(f64,f64)>,
    direction:&'static str,
    last_auto_move_time:time::Tm,
    food_location:(f64,f64),
    window_dimension:u32,
    previous_tail:(f64,f64),
    current_screen:&'static str,
    difficulty:usize
}

fn compare_ordered_pairs(a:(f64,f64),b:(f64,f64))->bool{
    let a_x=(a.0) as i64;
    let a_y=(a.1) as i64;
    let b_x=(b.0) as i64;
    let b_y=(b.1) as i64;
    (a_x,a_y)==(b_x,b_y)
}

fn generate_random_ordered_pair(grid_length:f64,snake_body:&Vec<(f64,f64)>,window_dimension:u32)->(f64,f64){
    println!("{:?}",grid_length);
    let between = Range::new(0, grid_length as i64);
    let mut rng = rand::thread_rng();
    let mut sum = 0;
    let mut tuple=(0.0,0.0);
    tuple.0=((between.ind_sample(&mut rng) as f64)/grid_length)*(window_dimension as f64);
    tuple.1=((between.ind_sample(&mut rng) as f64)/grid_length)*(window_dimension as f64);
    let mut under_snake_body=false;
    for ordered_pair in snake_body.iter(){
        if compare_ordered_pairs(tuple,*ordered_pair){
            under_snake_body=true;
            break;
        }
    }
    if under_snake_body{
        generate_random_ordered_pair(grid_length,snake_body,window_dimension)
    }else{
        tuple
    }

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

    fn render_game(&mut self, args: &RenderArgs){
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
        println!("{:?}",self.food_location);
        let window_dimension=self.window_dimension;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
            for tuple in snake_body.iter(){
                let transform = c.transform.rot_rad(rotation).trans(tuple.0,tuple.1);
                // Draw a box rotating around the middle of the screen.
                rectangle(RED, square, transform, gl);
            }
            let food_transform=c.transform.trans(food_location.0,food_location.1);
            ellipse(RED, food_ellipse, food_transform, gl);

        });
    }

    fn render(&mut self, args: &RenderArgs,ui:&mut conrod::Ui<opengl_graphics::glyph_cache::GlyphCache>) {
        if self.current_screen=="game_screen"{
            self.render_game(args)
        }else if self.current_screen=="start_screen"{
            self.render_start_screen(args,ui)
        }
    }

    fn render_start_screen(&mut self, args: &RenderArgs,ui:&mut conrod::Ui<opengl_graphics::glyph_cache::GlyphCache>){
        use graphics::*;
        const BLACK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const GREY:   [f32; 4] = [0.5, 0.5, 0.5, 1.0];
        let window_dimension=self.window_dimension as f64;
        let current_screen=&mut self.current_screen;
        let last_auto_move_time=&mut self.last_auto_move_time;
        let mut difficulty=&mut self.difficulty;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREY, gl);
            let mut font=opengl_graphics::glyph_cache::GlyphCache::new(Path::new("fonts/leaguegothic-regular-webfont.ttf")).unwrap();
            Split::new(MASTER).flow_down(&[
                Split::new(HEADER).color(light_green()),
                Split::new(BODY).color(blue()).length(window_dimension*0.75).pad_top(20.0),
            ]).set(ui);
            let dropdown_container=Floating::new();
            dropdown_container.label("Difficulty")
                .width(400.0)
                .show_title_bar(false)
                .mid_top_of(BODY)
                .color(grey())
                .pad_left(20.0)
                .set(DROPDOWNCONTAINER, ui);
            Label::new("Welcome To Snake").color(dark_orange()).font_size(48).middle_of(HEADER).set(TITLE, ui);


            DropDownList::new(&mut vec!["Easy".to_string(),"Medium".to_string(),"Hard".to_string()],&mut Some(*difficulty))
                .enabled(true)
                .middle_of(DROPDOWNCONTAINER)
                .dimensions(120.0, 40.0)
                .react(|selected_idx: &mut Option<usize>, new_idx, string| {
                    *difficulty = new_idx;
                    *selected_idx = Some(new_idx);
                    })
                .set(DROPDOWN,ui);
            Label::new("Difficulty").color(dark_orange()).font_size(24).middle_of(HEADER).mid_left_of(DROPDOWNCONTAINER).set(DROPDOWNLABEL, ui);

            Button::new()
                        .color(conrod::color::red())
                        .dimensions(120.0, 60.0)
                        .label("Start Game")
                        .react(|| {
                            println!("hi");
                            *current_screen="game_screen";
                            *last_auto_move_time=time::now();
                        })
                        .middle_of(BODY)
                        .set(STARTBUTTON, ui);

            ui.draw(c,gl);

        });
    }


    fn move_square(&mut self,press_args:input::Button)->(){
        println!("move_square");
        self.previous_tail=self.snake_body[self.snake_body.len()-1];
        let snake_width=self.snake_width;
        let mut new_snake_body=vec![];
        move_up_all_but_first(&self.snake_body,&mut new_snake_body);
        match press_args{
            input::Button::Keyboard(key)=>{if key==input::keyboard::Key::Right{
                    if !(self.direction=="left"){
                        new_snake_body[0].0=self.snake_body[0].0+snake_width;
                        new_snake_body[0].1=self.snake_body[0].1;
                        self.direction="right";
                        self.last_auto_move_time=time::now();
                        self.snake_body=new_snake_body;
                    }else{
                        return ()
                    }
                }else if key==input::keyboard::Key::Left{
                    if !(self.direction=="right"){
                        new_snake_body[0].0=self.snake_body[0].0-snake_width;
                        new_snake_body[0].1=self.snake_body[0].1;
                        self.direction="left";
                        self.last_auto_move_time=time::now();
                        self.snake_body=new_snake_body;
                    }else{
                        return ()
                    }
                    }else if key==input::keyboard::Key::Up{
                        if !(self.direction=="down"){
                            new_snake_body[0].1=self.snake_body[0].1-snake_width;
                            new_snake_body[0].0=self.snake_body[0].0;
                            self.direction="up";
                            self.last_auto_move_time=time::now();
                            self.snake_body=new_snake_body;
                        }else{
                            return ()
                        }
                        }else if key==input::keyboard::Key::Down{
                            if !(self.direction=="up"){
                                new_snake_body[0].1=self.snake_body[0].1+snake_width;
                                new_snake_body[0].0=self.snake_body[0].0;
                                self.direction="down";
                                self.last_auto_move_time=time::now();
                                self.snake_body=new_snake_body;
                            }else{
                                return ()
                            }
                        }
            },
            input::Button::Mouse(key)=>println!("{:?}",key)
        }

    }

    fn game_over(&mut self,r:RenderArgs)->bool{
        let snake_head=self.snake_body[0];
        if snake_head.0+self.snake_width>r.width as f64||snake_head.0<0.0||snake_head.1+self.snake_width>r.height as f64||snake_head.1<0.0{
            return true
        }
        for (i,tuple) in self.snake_body.iter().enumerate(){
            if i!=0{
                if compare_ordered_pairs(snake_head,*tuple){
                    return true
                }
            }
        }
        false
    }

    fn auto_move(&mut self)->(){
        let elapsed=time::now()-self.last_auto_move_time;
        let snake_width=self.snake_width;
        let duration_threshold=match self.difficulty{
            0=>time::Duration::milliseconds(1000),
            1=>time::Duration::milliseconds(500),
            2=>time::Duration::milliseconds(250),
            _=>time::Duration::milliseconds(1000)
        };
        if elapsed>duration_threshold{
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
        if compare_ordered_pairs(self.food_location,self.snake_body[0]){
            println!("eat_food");
            self.food_location=generate_random_ordered_pair((self.window_dimension as f64)/self.snake_width,&self.snake_body,self.window_dimension);
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
    let window_dimension=600;
    let snake_width=15.0;
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
        food_location:generate_random_ordered_pair((window_dimension as f64)/snake_width,&vec![(0.0,0.0)],window_dimension),
        previous_tail:(0.0,0.0),
        current_screen:"start_screen",
        difficulty:0
    };

    let ui=&mut conrod::Ui::new(opengl_graphics::glyph_cache::GlyphCache::new(Path::new("fonts/leaguegothic-regular-webfont.ttf")).unwrap(),conrod::Theme::default());
    for e in window.events() {
        ui.handle_event(&e);
        match e.press_args(){
            Some(press_args)=>app.move_square(press_args),
            None=>()
        }
        //println!("{:?}",e.render_args());
        //println!("{:?}",e.update_args());
        if let Some(r) = e.render_args() {
          app.render(&r,ui);
          if app.game_over(r){
              println!("{:?}","You Lose!");
              println!("{:?}",r);
              break;
          }
        }

        //if let Some(u) = e.update_args() {
        //    app.update(&u);
        //}
    }
}

const MASTER: CanvasId = 0;
const HEADER: CanvasId = MASTER + 1;
const BODY: CanvasId = HEADER + 1;
const DROPDOWNCONTAINER: CanvasId = BODY + 1;

const TITLE: WidgetId = 0;
const SUBTITLE: WidgetId = TITLE + 1;
const STARTBUTTON: WidgetId = SUBTITLE + 1;
const DROPDOWN: WidgetId = STARTBUTTON + 1;
const DROPDOWNLABEL: WidgetId = DROPDOWN + 1;
