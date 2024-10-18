use crate::Limiters::{Price, Quality};
use crate::Options::{Entry, Exit, Voice};
use std::mem::swap;

#[derive(Debug, PartialEq)]
pub enum Options {
    Exit = 0,
    Voice = 1,
    Entry = 2,
}

#[derive(Debug, PartialEq)]
pub enum Limiters {
    Quality = 0,
    Price = 1,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Org {
    pub allows: [bool; 2], // to be accessed using the Options enum
    pub cost: [u32; 3],    // to be accessed using the Options enum
    pub quality: u32,
    pub price: u32,
}

impl Org {
    pub fn is_declining(&self, limiters: [u32; 2], relevant: &[Limiters; 2]) -> bool {
        let quality_declining = self.quality < limiters[Quality as usize];
        let price_declining = self.price > limiters[Price as usize];

        if !relevant.contains(&Price) {
            quality_declining
        } else if relevant.contains(&Quality) && relevant.contains(&Price) {
            quality_declining && price_declining
        } else {
            price_declining
        }
    }

    fn tolerate_decline(
        &self,
        limiters: [u32; 2],
        relevant: &[Limiters; 2],
        tolerance: [u32; 2],
    ) -> bool {
        let quality_tolerated =
            self.quality >= (limiters[Quality as usize] - tolerance[Quality as usize]);
        let price_tolerated = self.price <= (limiters[Price as usize] - tolerance[Price as usize]);

        if !relevant.contains(&Price) {
            return quality_tolerated;
        } else if relevant.contains(&Price) {
            return quality_tolerated && price_tolerated;
        }
        price_tolerated
    }

    fn can_use(
        &self,
        max_cost: u32,
        option: Options,
        influence: Option<u32>,
        elastic: Option<bool>,
        alt_exists: Option<bool>,
    ) -> bool {
        match option {
            Exit => {
                self.cost[0] <= max_cost
                    && self.allows[0]
                    && (elastic.unwrap() || alt_exists.unwrap())
            }
            Voice => self.cost[1] <= max_cost && self.allows[1] && influence.unwrap() > 0,
            Entry => self.cost[2] <= max_cost,
        }
    }
}

pub struct Membership {
    pub org: Option<Org>,
    pub alternatives: Vec<Org>,
    pub relevant: [Limiters; 2],
    pub maximize: Limiters,
    pub limiters: [u32; 2],  // to be accessed using the Limiters enum
    pub tolerance: [u32; 2], // to be accessed using the Limiters enum
    pub max_cost: [u32; 3],  // to be accessed using the Options enum
    pub elastic: bool,
    pub influence: u32,
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
                    && a.quality >= self.limiters[Quality as usize]
            })
            .min_by_key(|a| a.cost[Entry as usize])
    }
}

pub struct Member {
    pub org_vec: Vec<Membership>,
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

    pub fn decision_making(m: &mut Membership) -> Option<Options> {
        if let Some(org) = &m.org {
            let decline_tolerated = org.tolerate_decline(m.limiters, &m.relevant, m.tolerance);

            // if the quality of an organization is below the minimum accepted quality...
            if org.is_declining(m.limiters, &m.relevant) {
                let alt_exists = m.get_best_alt().is_some();
                let can_use_exit = org.can_use(
                    m.get_max_cost(Exit),
                    Exit,
                    None,
                    Some(m.elastic),
                    Some(alt_exists),
                );

                // ...and voice can be used...
                if org.can_use(m.get_max_cost(Voice), Voice, Some(m.influence), None, None) {
                    // if exit cannot be used or decline is tolerable
                    if !can_use_exit || decline_tolerated {
                        Member::voice(m); // voice will be used
                        return Some(Voice);
                    }
                } else if can_use_exit && !decline_tolerated {
                    // else, if exit can be used and decline is not tolerable, exit will be used
                    Member::exit(m);
                    return Some(Exit);
                }
            }
            return None;
        }
        None
    }

    pub fn check(&mut self) {
        for m in self.org_vec.iter_mut() {
            Member::decision_making(m);
        }
    }
}
