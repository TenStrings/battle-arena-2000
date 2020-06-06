use crate::ComponentManager;
use crate::Entity;
use nalgebra_glm as glm;

#[derive(Default)]
pub struct CollisionSystem {}

#[derive(Clone, Debug)]
pub struct CollisionComponent {
    pub radius: f32,
}

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem {}
    }

    pub fn run(
        &self,
        components: &mut ComponentManager,
        mut on_collision: impl FnMut(Entity, Entity) -> (),
    ) {
        let len = components.collision.len();
        for (index1, collision) in components.collision[..len - 1].iter().enumerate() {
            if let Some(collision1) = collision {
                let pos1 = components.position[index1]
                    .as_ref()
                    .expect("collision object doesn't have a position")
                    .clone()
                    .into();

                let body1 = components.body[index1]
                    .as_ref()
                    .expect("collision object doesn't have body");

                let v1 = body1.velocity;
                let m1 = body1.mass;

                for (index2, collision_other) in
                    components.collision.iter().enumerate().skip(index1 + 1)
                {
                    match collision_other {
                        Some(collision2) if index1 != index2 => {
                            let pos2 = components.position[index2]
                                .as_ref()
                                .expect("collision object doesn't have a position")
                                .clone()
                                .into();

                            let body2 = components.body[index2]
                                .as_ref()
                                .expect("collision object doesn't have body");
                            let v2 = body2.velocity;
                            let m2 = body2.mass;

                            let distance2 = glm::distance2(&pos1, &pos2);

                            if distance2 < (collision1.radius + collision2.radius).powf(2.0) {
                                let c1 = glm::vec2(pos1.x.into(), pos1.y.into());
                                let c2 = glm::vec2(pos2.x.into(), pos2.y.into());
                                let r1 = collision1.radius;
                                let r2 = collision2.radius;

                                let (p1, p2) = get_collision_points(c1, c2, v1, v2, r1, r2);

                                let new_pos1 = components.position[index1].as_mut().unwrap();

                                new_pos1.set_x_wrap(p1.x as f32);
                                new_pos1.set_y_wrap(p1.y as f32);

                                let new_pos2 = components.position[index2].as_mut().unwrap();

                                new_pos2.set_x_wrap(p2.x as f32);
                                new_pos2.set_y_wrap(p2.y as f32);

                                let collision_direction = (p2 - p1) / glm::distance(&p2, &p1);

                                let cu1 = glm::dot(&v1, &collision_direction);
                                let cu2 = glm::dot(&v2, &collision_direction);

                                let cv1 = cu1 * (m1 - m2) + 2.0 * m2 * cu2;
                                let cv2 = cu2 * (m2 - m1) + 2.0 * m1 * cu1;

                                let m = m1 + m2;

                                // XXX: I have no idea why do I need those abs()?
                                components.body[index1].as_mut().unwrap().velocity -=
                                    collision_direction * cv1.abs() / m;
                                components.body[index2].as_mut().unwrap().velocity +=
                                    collision_direction * cv2.abs() / m;

                                on_collision(Entity(index1 as u32), Entity(index2 as u32))
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
}

impl CollisionComponent {
    pub fn new(radius: f32) -> CollisionComponent {
        CollisionComponent { radius }
    }
}

enum QuadraticSolution {
    None,
    One(f64),
    Two(f64, f64),
}

fn solve_quadratic(a: f64, b: f64, c: f64) -> QuadraticSolution {
    let discriminant = b.powf(2.0) - 4.0 * a * c;
    match discriminant {
        // TODO: check this (I think the == it doesn't matter for 0.0)
        x if x == 0.0 => QuadraticSolution::One(-0.5 * b / a),
        x if x.is_sign_negative() => QuadraticSolution::None,
        d => {
            let sqrt = d.sqrt();
            let div = 2.0 * a;
            QuadraticSolution::Two((-b + sqrt) / div, (-b - sqrt) / div)
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn get_collision_points(
    c1: glm::DVec2,
    c2: glm::DVec2,
    v1: glm::DVec2,
    v2: glm::DVec2,
    r1: f32,
    r2: f32,
) -> (glm::DVec2, glm::DVec2) {
    // relative position of c2 with respect to c1
    let p = c2 - c1;
    // relative velocity of c2 with respect to c1
    let v = v2 - v1;
    // the quadratic terms
    let a: f64 = glm::magnitude2(&v);
    let b: f64 = glm::dot(&v, &p) * 2.0;
    let c: f64 = glm::magnitude2(&p) - f64::from((r1 + r2).powf(2.0));

    match solve_quadratic(a, b, c) {
        QuadraticSolution::Two(root1, root2) => {
            let t = if root1 < root2 { root1 } else { root2 };

            let solution2 = p + t * v + c1;
            let solution1 = solution2 - (p / glm::length(&p)) * f64::from(r1 + r2);

            (solution1, solution2)
        }
        QuadraticSolution::One(t) => {
            let solution = p + t * v;

            (solution + c2, solution + c1)
        }
        QuadraticSolution::None => unreachable!("there should be a solution"),
    }
}
