use crate::lib::{
    time::time::Time,
    units::unit::{Power, Unit, UnitPos},
};
use dyn_clone::DynClone;
use std::fmt::Debug;

dyn_clone::clone_trait_object!(Bonus);
pub trait Bonus: DynClone + Debug + Send {
    fn on_attacked(
        &self,
        damage: Power,
        receiver: &mut Unit,
        sender: &mut Unit,
        receiver_pos: UnitPos,
        sender_pos: UnitPos,
    ) -> Power {
        damage
    }
    fn on_attacking(&self, damage: Power, receiver: &mut Unit, sender: &mut Unit) -> Power {
        damage
    }
    fn on_kill(&self, receiver: &mut Unit, sender: &mut Unit) -> bool {
        false
    }
    fn on_tick(&self, unit: &mut Unit) -> bool {
        false
    }
    fn on_hour(&self, unit: &mut Unit, time: Time) -> bool {
        false
    }
    fn on_battle_start(&self, unit: &mut Unit) -> bool {
        false
    }
    fn on_move_skip(&self, unit: &mut Unit) -> bool {
        false
    }
    fn id(&self) -> &'static str;
}
