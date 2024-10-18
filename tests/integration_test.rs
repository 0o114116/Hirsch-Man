use Hirsch_Man::{
    Limiters::Quality,
    Member, Membership,
    Options::{Exit, Voice},
    Org,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_decline() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, true], // exit and voice are allowed
                cost: [0, 0, 0],
                quality: 2,
                price: 0,
            }),
            alternatives: vec![Org {
                // an equally good alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
                price: 0,
            }], // for both organizations:
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],
            max_cost: [1, 1, 1], // exit, voice, and entry to the alternative are viable
            elastic: true,       // the demand is elastic and therefore exit can be used
            // even without a viable alternative
            influence: 1,
        };

        assert_eq!(m.org.unwrap().is_declining(m.limiters, &m.relevant), false);
        // since the organization is not declining, no action should be taken
        //assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn declining_monopoly_1() {
        let mut m = Membership {
            org: Some(Org {
                allows: [false, false], // exit and voice are not allowed
                cost: [0, 0, 0],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![Org {
                // a better alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
                price: 0,
            }],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],
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
                allows: [true, false], // only exit is allowed
                cost: [0, 0, 0],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![], // no alternatives exist
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],   // ibid
            max_cost: [1, 1, 1], // exit is viable
            elastic: false,      // the demand is inelastic and therefore exit cannot be used
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
                allows: [true, true], // exit and voice are allowed
                cost: [2, 2, 0],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![Org {
                // a better alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 2,
                price: 0,
            }],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],
            max_cost: [1, 1, 1], // exit and voice are too expensive
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
                allows: [true, false], // only exit is allowed
                cost: [1, 1, 1],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![Org {
                // a better alternative exists
                allows: [true, true],
                cost: [0, 0, 3],
                quality: 2,
                price: 0,
            }],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],
            max_cost: [2, 2, 2], // exit is viable but entry to the alternative isn't
            elastic: false,      // the demand is inelastic
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
                allows: [true, false], // only exit is allowed
                cost: [1, 1, 1],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![Org {
                // an equally bad alternative exists
                allows: [true, true],
                cost: [0, 0, 0],
                quality: 1,
                price: 0,
            }],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],   // ibid
            max_cost: [2, 2, 2], // exit is viable
            elastic: false,      // the demand is inelastic
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
                allows: [false, true], // only voice is allowed
                cost: [0, 0, 0],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],   // ibid
            max_cost: [1, 1, 1], // voice is viable
            elastic: true,
            influence: 0, // influence is null
        };

        // since exit is not allowed and influence is seen as null, no action should be taken
        assert_eq!(Member::decision_making(&mut m), None)
    }

    #[test]
    fn monopoly_1() {
        let mut m = Membership {
            org: Some(Org {
                allows: [false, true], // only voice is allowed
                cost: [0, 0, 0],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],
            max_cost: [1, 1, 1], // voice is viable
            elastic: true,
            influence: 1, // influence exists
        };

        // since exit is not allowed but voice is completely viable, voice will be used
        assert_eq!(Member::decision_making(&mut m), Some(Voice))
    }

    #[test]
    fn monopoly_2() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, true], // exit and voice are allowed
                cost: [0, 0, 0],
                quality: 2,
                price: 0,
            }),
            alternatives: vec![],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [3, 0],
            tolerance: [1, 0],   // but decline is tolerable
            max_cost: [1, 1, 1], // exit and voice are viable
            elastic: true,
            influence: 1, // influence exists
        };

        // since decline is tolerable and voice is viable, voice will be used
        assert_eq!(Member::decision_making(&mut m), Some(Voice))
    }

    #[test]
    fn monopoly_3() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, true], // exit and voice are allowed
                cost: [0, 0, 0],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],   // ibid
            max_cost: [1, 1, 1], // exit and voice are viable
            elastic: false,      // the demand is inelastic
            influence: 1,        // influence exists
        };

        // since demand is inelastic and no alternative exists, but voice is viable, voice will be
        // used
        assert_eq!(Member::decision_making(&mut m), Some(Voice))
    }

    #[test]
    fn elastic_exit() {
        let mut m = Membership {
            org: Some(Org {
                allows: [true, false], // exit is allowed
                cost: [0, 0, 0],
                quality: 1,
                price: 0,
            }),
            alternatives: vec![],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],   // ibid
            max_cost: [1, 1, 1], // exit is viable
            elastic: true,       // the demand is elastic
            influence: 0,
        };

        // since demand is elastic and only exit is allowed and viable, exit will be used
        assert_eq!(Member::decision_making(&mut m), Some(Exit))
    }

    #[test]
    fn inelastic_exit_with_single_alt() {
        let org = Org {
            allows: [true, false],
            cost: [1, 1, 1],
            quality: 1,
            price: 0,
        };
        let alt = Org {
            allows: [true, true],
            cost: [1, 1, 1],
            quality: 2,
            price: 0,
        };

        let mut m = Membership {
            org: Some(org.clone()),          // exit is allowed
            alternatives: vec![alt.clone()], // a better alternative exists
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],   // ibid
            max_cost: [2, 2, 2], // exit is viable, and so is entry to alt
            elastic: false,      // the demand is inelastic
            influence: 0,
        };

        // since demand is inelastic, only exit is allowed and viable, and an acceptable alternative
        // exists, exit will be used
        assert_eq!(Member::decision_making(&mut m), Some(Exit));
        assert_eq!(m.org, Some(alt));
        assert!(m.alternatives.contains(&org));
    }

    #[test]
    fn inelastic_exit_with_many_alts() {
        let org = Org {
            allows: [true, false], // exit is allowed
            cost: [1, 1, 1],
            quality: 1,
            price: 0,
        };
        let best_alt = Org {
            allows: [true, true],
            cost: [1, 1, 1],
            quality: 2,
            price: 0,
        };

        let mut m = Membership {
            org: Some(org.clone()),
            alternatives: vec![
                // several good alternatives exist
                best_alt.clone(), // one of which has the lowest entry cost
                Org {
                    allows: [true, true],
                    cost: [1, 1, 2],
                    quality: 2,
                    price: 0,
                },
            ],
            relevant: [Quality, Quality],
            maximize: Quality,
            limiters: [2, 0],
            tolerance: [0, 0],   // ibid
            max_cost: [2, 2, 2], // exit is viable, and so is entry to the alternatives
            elastic: false,      // demand is inelastic
            influence: 0,
        };

        // since demand is inelastic, only exit is allowed and viable, and many acceptable
        // alternatives exist, exit will be used to choose the alternative with the lowest cost
        assert_eq!(Member::decision_making(&mut m), Some(Exit));
        assert_eq!(m.org, Some(best_alt));
        assert!(m.alternatives.contains(&org));
    }
}
