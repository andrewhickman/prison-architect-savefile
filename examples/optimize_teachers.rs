use std::collections::HashMap;

use prison_architect_savefile::Node;

fn main() -> anyhow::Result<()> {
    let mut prison = prison_architect_savefile::read(r#"cloudsaves\felrock2.prison"#)?;

    let rooms = resolve_rooms(&prison);
    let mut foremen = resolve_offices(&prison, &rooms, "Foreman");
    let mut psychologists = resolve_offices(&prison, &rooms, "Psychologist");

    let programs = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\prison-architect\main\data\reform_programs.txt"#,
    )?;
    let programs_dlc = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\prison-architect\main\data\reform_programs_dlc.txt"#,
    )?;
    let definitions: HashMap<&str, &Node> = programs
        .children()
        .chain(programs_dlc.children())
        .map(|(_, program)| (program.property("Name").unwrap(), program))
        .collect();

    for (_, program) in prison.child_mut("Reform").unwrap().child_mut("Programs").unwrap().children_mut() {
        let definition = definitions[program.property("Type").unwrap()];

        let offices = match definition.property("Teacher") {
            Some("Foreman") => &mut foremen,
            Some("Psychologist") => &mut psychologists,
            _ => continue,
        };

        let program_room: u32 = program.property("Room.u").unwrap().parse().unwrap();
        let program_location = rooms[&program_room];
        let closest_office = offices.iter_mut().min_by_key(|office| program_location.dist(&office.location) as u32).unwrap();

        program.set_property("Teacher.i", closest_office.teacher_i.to_string());
        program.set_property("Teacher.u", closest_office.teacher_u.to_string());
        if program.property("Error") == Some("NoTeacher") {
            program.set_property("Error", "None");
        }

        closest_office.programs.push(program.clone());
    }

    for room in foremen.iter().chain(&psychologists) {
        println!("* {} {}", room.ty, room.teacher_u);
        for program in &room.programs {
            println!("{} {}", program.property("StartHour").unwrap(), program.property("Type").unwrap());
        }
        println!();
    }

    prison.write(r#"saves\after.prison"#)?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}

pub struct Office {
    ty: String,
    teacher_i: u32,
    teacher_u: u32,
    location: Point,
    programs: Vec<Node>,
}

impl Point {
    pub fn center(points: impl IntoIterator<Item = Point>) -> Point {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut count = 0;

        for point in points {
            x += point.x;
            y += point.y;
            count += 1;
        }

        let count = count as f64;
        Point { x: x / count, y: y / count }
    }

    pub fn dist(&self, other: &Self) -> f64 {
        f64::hypot(self.x - other.x, self.y - other.y)
    }
}

fn resolve_rooms(prison: &Node) -> HashMap<u32, Point> {
    let mut cells: HashMap<u32, Vec<Point>> = HashMap::new();

    for (point, cell) in prison.child("Cells").unwrap().children() {
        if let Some(room) = cell.property("Room.u") {
            let room: u32 = room.parse().unwrap();

            let (x, y) = point.split_once(' ').unwrap();
            let (x, y) = (x.parse().unwrap(), y.parse().unwrap());

            cells.entry(room).or_default().push(Point{ x, y})
        }
    }

    cells.into_iter()
        .map(|(room, points)| (room, Point::center(points)))
        .collect()
}

fn resolve_offices(prison: &Node, rooms: &HashMap<u32, Point>, ty: &str) -> Vec<Office> {
    prison.child("Objects").unwrap().children()
        .filter(|(_, child)| child.property("Type") == Some(ty) )
        .map(|(_, child)| {
            let teacher_i: u32 = child.property("Id.i").unwrap().parse().unwrap();
            let teacher_u: u32 = child.property("Id.u").unwrap().parse().unwrap();
            let room_u: u32 = child.property("Office.u").unwrap().parse().unwrap();
            let location = rooms[&room_u];

            Office { ty: ty.to_string(), teacher_i, teacher_u, location, programs: Vec::new() }
        })
        .collect()
}
