use crate::Options::{Entry, Exit, Voice};
use std::mem::swap;

#[derive(Debug)]
#[derive(PartialEq)]
enum Options {
    Exit = 0,
    Voice = 1,
    Entry = 2,
}

struct Org {
    allows: [bool; 2],
    cost: [u32; 3],
    quality: u32,
}

impl Org {
    fn is_declining(&self, min_accepted_quality: u32) -> bool {
        self.quality <= min_accepted_quality
    }

    fn tolerate_decline(&self, min_accepted_quality: u32, tolerance: u32) -> bool {
        self.quality >= (min_accepted_quality - tolerance)
    }

    fn can_use(&self, max_cost: u32, option: Options) -> bool {
        match option {
            Exit => self.cost[0] <= max_cost && self.allows[0],
            Voice => self.cost[1] <= max_cost && self.allows[1],
            Entry => self.cost[2] <= max_cost,
        }
    }
}

struct Membership {
    org: Option<Org>,
    alternatives: Vec<Org>,
    min_accepted_quality: u32,
    tolerance: u32,
    max_cost: [u32; 3],
    elastic: bool,
    belief_in_voice: bool,
    influence: u32,
}

impl Membership {
    fn get_max_cost(&self, option: Options) -> u32 {
        self.max_cost[option as usize]
    }

    fn get_best_alt(&self) -> Option<&Org> {
        self.alternatives
            .iter()
            .filter(|a| {
                a.cost[Entry as usize] <= self.get_max_cost(Entry)
                    && a.quality >= self.min_accepted_quality
            })
            .min_by_key(|a| a.cost[Entry as usize])
    }
}

struct Member {
    org_vec: Vec<Membership>,
}

impl Member {
    fn voice(m: &mut Membership) {
        if let Some(ref mut org) = m.org {
            org.quality += m.influence;
        }
    }

    fn exit(m: &mut Membership) {
        let mut has_switched = false;

        if let Some(best_alt) = m.get_best_alt() {
            if let Some(index) = m
                .alternatives
                .iter()
                .position(|alt| alt.quality == best_alt.quality && alt.cost == best_alt.cost)
            {
                swap(m.org.as_mut().unwrap(), &mut m.alternatives[index]);
                has_switched = true;
            }
        }

        if !has_switched {
            m.alternatives.push(m.org.take().unwrap());
        }
    }

    fn decision_making(m: &mut Membership) -> Option<Options> {
        if let Some(org) = &m.org {
            let can_use_exit = org.can_use(m.get_max_cost(Exit), Exit);
            let decline_tolerated = org.tolerate_decline(m.min_accepted_quality, m.tolerance);

            if org.is_declining(m.min_accepted_quality) {
                let alt_exists = m.get_best_alt().is_some();

                if org.can_use(m.get_max_cost(Voice), Voice) {
                    if !can_use_exit
                        || m.belief_in_voice
                        || decline_tolerated
                        || (!m.elastic && !alt_exists)
                    {
                        Member::voice(m);
                        return Some(Voice);
                    }
                } else if can_use_exit && !decline_tolerated && (m.elastic || alt_exists) {
                    Member::exit(m);
                    return Some(Exit);
                }
            }
            return None;
        }
        None
    }

    fn check(&mut self) {
        for m in self.org_vec.iter_mut() {
            Member::decision_making(m);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_decline() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
            }),
            alternatives: vec![Org {
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
            }],
            min_accepted_quality: 1,
            tolerance: 0,
            max_cost: [1, 1, 1],
            elastic: true,
            belief_in_voice: true,
            influence: 1,
        };

        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn monopoly_1() {
        let mut m = Membership {
            org: Some(Org {
                allows: [false, false],
                cost: [0, 0, 0],
                quality: 1,
            }),
            alternatives: vec![Org {
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
            }],
            min_accepted_quality: 2,
            tolerance: 0,
            max_cost: [1, 1, 1],
            elastic: true,
            belief_in_voice: false,
            influence: 0,
        };

        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn monopoly_2() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, false],
                cost: [0, 0, 0],
                quality: 1,
            }),
            alternatives: vec![],
            min_accepted_quality: 2,
            tolerance: 0,
            max_cost: [1, 1, 1],
            elastic: false,
            belief_in_voice: false,
            influence: 0,
        };

        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn monopoly_3() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, true],
                cost: [2, 2, 0],
                quality: 1,
            }),
            alternatives: vec![Org {
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
            }],
            min_accepted_quality: 2,
            tolerance: 0,
            max_cost: [1, 1, 1],
            elastic: true,
            belief_in_voice: true,
            influence: 1,
        };

        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn monopoly_4() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, false],
                cost: [1, 1, 1],
                quality: 1,
            }),
            alternatives: vec![Org {
                allows: [true, true],
                cost: [0, 0, 3],
                quality: 2,
            }],
            min_accepted_quality: 2,
            tolerance: 0,
            max_cost: [2, 2, 2],
            elastic: false,
            belief_in_voice: false,
            influence: 0,
        };

        assert_eq!(Member::decision_making(&mut m), None)
    }
}
