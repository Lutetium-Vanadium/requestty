enum Direction {
    Forward,
    Right,
    Left,
    Back,
}

fn prompt() -> discourse::Result<Direction> {
    let answer = discourse::prompt_one(
        discourse::Question::select("direction")
            .message("Which direction would you like to go?")
            .choice("Forward")
            .choice("Right")
            .choice("Left")
            .choice("Back"),
    )?;

    match answer.as_list_item().unwrap().index {
        0 => Ok(Direction::Forward),
        1 => Ok(Direction::Right),
        2 => Ok(Direction::Left),
        3 => Ok(Direction::Back),
        _ => unreachable!(),
    }
}

fn main() {
    println!("You find yourself in a small room, there is a door in front of you.");

    if let Err(e) = exit_house() {
        println!("{:?}", e);
    }
}

fn exit_house() -> discourse::Result<()> {
    loop {
        match prompt()? {
            Direction::Forward => {
                println!("You find yourself in a forest");
                println!("There is a wolf in front of you; a friendly looking dwarf to the right and an impasse to the left.");

                break encounter1();
            }
            _ => println!("You cannot go that way"),
        }
    }
}

fn encounter1() -> discourse::Result<()> {
    loop {
        match prompt()? {
            Direction::Forward => {
                println!("You attempt to fight the wolf");
                println!("Theres a stick and some stones lying around you could use as a weapon");

                break encounter2b();
            }
            Direction::Right => {
                println!("You befriend the dwarf");
                println!("He helps you kill the wolf. You can now move forward");

                break encounter2a();
            }
            _ => println!("You cannot go that way"),
        }
    }
}

fn encounter2a() -> discourse::Result<()> {
    loop {
        match prompt()? {
            Direction::Forward => {
                println!(
                    r"You find a painted wooden sign that says:
 ____  _____  ____  _____
(_  _)(  _  )(  _ \(  _  )
  )(   )(_)(  )(_) ))(_)(
 (__) (_____)(____/(_____)"
                );

                break Ok(());
            }
            _ => println!("You cannot go that way"),
        }
    }
}

fn encounter2b() -> discourse::Result<()> {
    discourse::prompt_one(
        discourse::Question::select("weapon")
            .message("Pick one")
            .choice("Use the stick")
            .choice("Grab a large rock")
            .choice("Try and make a run for it")
            .choice("Attack the wolf unarmed"),
    )?;

    println!("The wolf mauls you. You die. The end.");

    Ok(())
}
