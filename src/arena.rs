use crate::PositionComponent;

pub struct Arena {
    pub percent: f32,
}

impl Arena {
    pub fn new() -> Arena {
        Arena { percent: 1.0f32 }
    }

    pub fn shrink(&mut self, amount: f32) {
        let new_percentage = self.percent - amount;
        if new_percentage >= 0.0 {
            self.percent = new_percentage;
        } else {
            self.percent = 0.0;
        }
    }

    pub fn contains(&self, position: &PositionComponent) -> bool {
        let good_width = crate::X_MAX * self.percent;
        let good_height = crate::Y_MAX * self.percent;

        let x_thresh = (crate::X_MAX - good_width) / 2.0;
        let y_thresh = (crate::Y_MAX - good_height) / 2.0;

        position.x > x_thresh
            && position.x < (crate::X_MAX - x_thresh)
            && position.y > y_thresh
            && position.y < (crate::Y_MAX - y_thresh)
    }
}

impl Default for Arena {
    fn default() -> Arena {
        Arena::new()
    }
}
