use crate::Options::{Entry, Exit, Voice};
use std::mem::swap;

#[derive(Debug)]
#[derive(PartialEq)]
enum Options {
    Exit = 0,
    Voice = 1,
    Entry = 2,
}

// TODO: difference between decline in quality and increase in price/cost of membership
struct Org {
    allows: [bool; 2],  // to be accessed using the Options enum
    cost: [u32; 3],     // to be accessed using the Options enum
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
    influence: u32,
}

impl Membership {
    fn get_max_cost(&self, option: Options) -> u32 {
        self.max_cost[option as usize]
    }

    // finds the alternative Org with the lowest entry cost among those with acceptable quality
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

        // if an acceptable alternative exists, the original organization will be moved to the
        // alternatives vector, and the chosen alternative will replace the original organization
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

        // if no acceptable alternative exists, exit will be used and the original organization
        // will be moved to the alternatives vector
        if !has_switched {
            m.alternatives.push(m.org.take().unwrap());
        }
    }

    fn decision_making(m: &mut Membership) -> Option<Options> {
        if let Some(org) = &m.org {
            let can_use_exit = org.can_use(m.get_max_cost(Exit), Exit);
            let decline_tolerated = org.tolerate_decline(m.min_accepted_quality, m.tolerance);

            // if the quality of an organization is below the minimum accepted quality...
            if org.is_declining(m.min_accepted_quality) {
                let alt_exists = m.get_best_alt().is_some();

                // TODO: move considerations of elasticity and availability of alternatives to
                //  can_use_exit and move considerations of influence to can_use_voice
                if org.can_use(m.get_max_cost(Voice), Voice) {  // and voice can be used
                    if (!can_use_exit                           // and exit cannot
                        || decline_tolerated                    // or the decline is tolerable
                        || (!m.elastic && !alt_exists))         // or demand is inelastic and no
                                                                // acceptable alternative exists
                        && m.influence > 0                      // and influence is not null
                    {
                        Member::voice(m);                       // voice will be used
                        return Some(Voice);
                    }
                } else if can_use_exit && !decline_tolerated && (m.elastic || alt_exists) {
                    // else, if exit can be used and decline is not tolerable and demand is either
                    // elastic or an alternative exists, exit will be used
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
                allows: [true, true],   // exit and voice are allowed
                cost: [0, 0, 0],
                quality: 2,
            }),
            alternatives: vec![Org {    // an equally good alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
            }],                         // for both organizations:
            min_accepted_quality: 1,    // the quality is acceptable
            tolerance: 0,               // ibid
            max_cost: [1, 1, 1],        // exit, voice, and entry to the alternative are viable
            elastic: true,              // the demand is elastic and therefore exit can be used
                                        // even without a viable alternative
            influence: 1,
        };

        // since the organization is not declining, no action should be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn declining_monopoly_1() {
        let mut m = Membership {
            org: Some(Org {
                allows: [false, false], // exit and voice are not allowed
                cost: [0, 0, 0],
                quality: 1,
            }),
            alternatives: vec![Org {    // a better alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
            }],
            min_accepted_quality: 2,    // the organization is declining
            tolerance: 0,               // ibid
            max_cost: [1, 1, 1],
            elastic: true,
            influence: 1,
        };

        // since the organization does not allow neither exit nor voice, no action should be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn declining_monopoly_2() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, false],  // only exit is allowed
                cost: [0, 0, 0],
                quality: 1,
            }),
            alternatives: vec![],       // no alternatives exist
            min_accepted_quality: 2,    // the organization is declining
            tolerance: 0,               // ibid
            max_cost: [1, 1, 1],        // exit is viable
            elastic: false,             // the demand is inelastic and therefore exit cannot be used
                                        // without a viable alternative
            influence: 0,
        };

        // since demand is inelastic and no alternatives exist, no action will be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn declining_monopoly_3() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, true],   // exit and voice are allowed
                cost: [2, 2, 0],
                quality: 1,
            }),
            alternatives: vec![Org {    // a better alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
            }],
            min_accepted_quality: 2,    // the organization is declining
            tolerance: 0,               // ibid
            max_cost: [1, 1, 1],        // exit and voice are too expensive
            elastic: true,
            influence: 1,
        };

        // since exit and voice are too expensive, no action should be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn declining_monopoly_4() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, false],  // only exit is allowed
                cost: [1, 1, 1],
                quality: 1,
            }),
            alternatives: vec![Org {    // a better alternative exists
                allows: [true, true],
                cost: [0, 0, 3],
                quality: 2,
            }],
            min_accepted_quality: 2,    // the organization is declining
            tolerance: 0,               // ibid
            max_cost: [2, 2, 2],        // exit and voice are viable but entry to the alternative
                                        // isn't
            elastic: false,             // the demand is inelastic
            influence: 0,
        };

        // since demand is inelastic and entry to the only existing alternative is too expensive,
        // no action should be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn declining_monopoly_5() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, false],  // only exit is allowed
                cost: [1, 1, 1],
                quality: 1,
            }),
            alternatives: vec![Org {    // an equally bad alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 1,
            }],
            min_accepted_quality: 2,    // the organization is declining
            tolerance: 0,               // ibid
            max_cost: [2, 2, 2],        // exit and voice are viable
            // isn't
            elastic: false,             // the demand is inelastic
            influence: 0,
        };

        // since demand is inelastic and entry to the only existing alternative is not acceptable,
        // no action should be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn declining_monopoly_6() {
        let mut m = Membership {
            org: Some(Org {
                allows: [false, true],  // only voice is allowed
                cost: [0, 0, 0],
                quality: 1,
            }),
            alternatives: vec![],
            min_accepted_quality: 2,    // the organization is declining
            tolerance: 0,               // ibid
            max_cost: [1, 1, 1],        // voice is viable
            elastic: true,
            influence: 0,               // influence is null
        };

        // since exit is not allowed influence is perceived to be null, no action should be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    // TODO: write tests for monopolies that can recover through the use of voice, exit with elastic
    //  demand, exit with inelastic demand and a good alternative, exit with inelastic demand
    //  and several good alternatives, and exit with inelastic demand with alternatives of varying
    //  quality
}
