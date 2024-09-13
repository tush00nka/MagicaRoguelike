use bevy::prelude::*;

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ElementBar::default())
            .add_systems(Update, (fill_bar, cast_spell));
    }
}

#[derive(Debug, PartialEq)]
enum ElementType {
    Fire, Water, Earth, Air
}

#[derive(Resource)]
struct ElementBar {
    pub bar: Vec<ElementType>,
    pub max: i32,
}

impl ElementBar {
    fn clear(&mut self) {
        self.bar = vec![];
    }

    fn add(&mut self, element: ElementType) {
        if (self.bar.len() as i32) < self.max {
            self.bar.push(element);
        }
        else {
            println!("[I] Element bar is full!!");
        }
    }

    fn default() -> Self {
        ElementBar {
            bar: vec![],
            max: 2,
        }
    }
}



fn fill_bar(
    mut bar: ResMut<ElementBar>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        bar.add(ElementType::Fire);
        println!("{:?}", bar.bar);
    }

    if keyboard.just_pressed(KeyCode::Digit2) {
        bar.add(ElementType::Water);
        println!("{:?}", bar.bar);
    }

    if keyboard.just_pressed(KeyCode::Digit3) {
        bar.add(ElementType::Earth);
        println!("{:?}", bar.bar);
    }

    if keyboard.just_pressed(KeyCode::Digit4) {
        bar.add(ElementType::Air);
        println!("{:?}", bar.bar);
    }
}

fn cast_spell(
    mut bar: ResMut<ElementBar>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if bar.bar.contains(&ElementType::Fire) &&
           bar.bar.contains(&ElementType::Air) {
            println!("Casted Fireball!!");
        }
        else {
            println!("Casted Cringe Random Spell!");
        }

        bar.clear();
        println!("{:?}", bar.bar);
    }
}