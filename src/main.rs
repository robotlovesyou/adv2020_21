use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::rc::Rc;

fn main() {
    let all_foods = read_foods(include_str!("../input.txt").lines());
    println!("appearances: {}", part_1(&all_foods));
    println!("{}", part_2(&all_foods));
}

fn part_1(all_foods: &[Food]) -> usize {
    let safe = get_safe(&all_foods);
    let mut appearances = 0;
    for ingredient in safe {
        for food in all_foods.iter() {
            if food.ingredients.contains(&ingredient) {
                appearances += 1;
            }
        }
    }
    appearances
}

fn part_2(all_foods: &[Food]) -> String {
    let mut potential_allergens = all_potential_allergen_ingredients(all_foods);
    let mut allergens = reduce_allergens(&mut potential_allergens);
    allergens.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
    allergens
        .into_iter()
        .map(|(_, ingredient)| ingredient)
        .map(|ingredient| ingredient.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

struct Food {
    id: u32,
    ingredients: HashSet<Rc<str>>,
    allergens: HashSet<Rc<str>>,
}

impl Food {
    fn new(id: u32, ingredients: HashSet<Rc<str>>, allergens: HashSet<Rc<str>>) -> Food {
        Food {
            id,
            ingredients,
            allergens,
        }
    }
}

trait FoodOps {
    fn all_ingredients(&self) -> HashSet<Rc<str>>;

    fn all_allergens(&self) -> HashSet<Rc<str>>;

    fn first_with_allergen(&self, allergen: &Rc<str>) -> &Food;
}

impl FoodOps for &[Food] {
    fn all_ingredients(&self) -> HashSet<Rc<str>> {
        let mut ingredients = HashSet::new();
        for food in self.iter() {
            for ingredient in food.ingredients.iter() {
                ingredients.insert(ingredient.clone());
            }
        }
        ingredients
    }

    fn all_allergens(&self) -> HashSet<Rc<str>> {
        {
            let mut allergens = HashSet::new();
            for food in self.iter() {
                for allergen in food.allergens.iter() {
                    allergens.insert(allergen.clone());
                }
            }
            allergens
        }
    }

    fn first_with_allergen(&self, allergen: &Rc<str>) -> &Food {
        self.iter()
            .find(|food| food.allergens.contains(allergen))
            .expect("no food with given allergen")
    }
}

fn get_safe(all_foods: &[Food]) -> HashSet<Rc<str>> {
    let all_ingredients = all_foods.all_ingredients();

    let mut maybe_allergens = all_potential_allergen_ingredients(all_foods)
        .into_iter()
        .map(|(_, set)| set)
        .collect::<Vec<HashSet<Rc<str>>>>();

    let first = maybe_allergens.pop().expect("maybe allergens is empty");
    let danger_danger = maybe_allergens.iter().fold(first, |mut all, maybe| {
        maybe.iter().fold(&mut all, |set, a| {
            set.insert(a.clone());
            set
        });
        all
    });

    all_ingredients
        .difference(&danger_danger)
        .cloned()
        .collect::<HashSet<Rc<str>>>()
}

fn all_potential_allergen_ingredients(all_foods: &[Food]) -> Vec<(Rc<str>, HashSet<Rc<str>>)> {
    let all_allergens = all_foods.all_allergens();
    let mut maybe_allergens = Vec::new();

    for allergen_ref in all_allergens.iter() {
        let potential_allergens = potential_allergen_ingredients(all_foods, allergen_ref);
        maybe_allergens.push((allergen_ref.clone(), potential_allergens));
    }
    maybe_allergens
}

fn potential_allergen_ingredients(all_foods: &[Food], allergen: &Rc<str>) -> HashSet<Rc<str>> {
    let first_with_allergen = all_foods.first_with_allergen(allergen);
    let mut maybe_allergen_ingredient =
        first_with_allergen
            .ingredients
            .iter()
            .fold(HashSet::new(), |mut all, ingredient| {
                all.insert(ingredient.clone());
                all
            });

    for food in all_foods
        .iter()
        .filter(|f| f.allergens.contains(allergen) && f.id != first_with_allergen.id)
    {
        let new_maybe = maybe_allergen_ingredient
            .intersection(&food.ingredients)
            .fold(HashSet::new(), |mut all, ingredient| {
                all.insert(ingredient.clone());
                all
            });
        maybe_allergen_ingredient = new_maybe;
    }
    maybe_allergen_ingredient
}

fn reduce_allergens(potentials: &mut Vec<(Rc<str>, HashSet<Rc<str>>)>) -> Vec<(Rc<str>, Rc<str>)> {
    let mut discovered_allergens: HashSet<Rc<str>> = HashSet::new();
    let mut discovered_kinds: HashSet<Rc<str>> = HashSet::new();
    let mut paired = Vec::new();
    while discovered_allergens.len() < potentials.len() {
        for i in 0..potentials.len() {
            let (kind, potential) = potentials.get_mut(i).unwrap();
            if discovered_kinds.contains(kind) {
                continue;
            }
            for discovered in discovered_allergens.iter() {
                potential.remove(discovered);
            }
            if potential.len() == 1 {
                discovered_kinds.insert(kind.clone());
                let ingredient =
                    &(*potential.iter().collect::<Vec<&Rc<str>>>().first().unwrap()).clone();
                discovered_allergens.insert(ingredient.clone());
                paired.push((kind.clone(), ingredient.clone()));
            }
        }
    }
    paired
}

lazy_static! {
    static ref FOOD_REGEX: Regex =
        Regex::new(r"^(?P<ingredients>[\w\s]+)\(contains (?P<allergens>[\w\s,]+)\)$")
            .expect("illegal food regex");
}

fn read_foods<'a>(lines: impl Iterator<Item = &'a str>) -> Vec<Food> {
    let mut foods = Vec::new();

    let mut id = 0;
    for line in lines {
        if let Some(caps) = FOOD_REGEX.captures(line) {
            let ingredients =
                caps["ingredients"]
                    .split(' ')
                    .fold(HashSet::new(), |mut all, ingredient| {
                        if !ingredient.trim().is_empty() {
                            all.insert(Rc::from(ingredient));
                        }
                        all
                    });

            let allergens =
                caps["allergens"]
                    .split(", ")
                    .fold(HashSet::new(), |mut all, allergen| {
                        if !allergen.trim().is_empty() {
                            all.insert(Rc::from(allergen));
                        }
                        all
                    });

            foods.push(Food::new(id, ingredients, allergens));
            id += 1;
        }
    }

    foods
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)\n
trh fvjkl sbzzf mxmxvkd (contains dairy)\n
sqjhc fvjkl (contains soy)\n
sqjhc mxmxvkd sbzzf (contains fish)";

    #[test]
    fn it_correctly_finds_potential_allergen_ingredients() {
        let all_foods = read_foods(TEST_INPUT.lines());
        let potential_allergens = potential_allergen_ingredients(&all_foods, &Rc::from("fish"));
        assert_eq!(2, potential_allergens.len());
        assert!(potential_allergens.contains(&Rc::from("mxmxvkd")));
        assert!(potential_allergens.contains(&Rc::from("sqjhc")));
    }

    #[test]
    fn it_counts_the_number_of_safe_ingredients() {
        let mut all_foods = read_foods(TEST_INPUT.lines());
        let safe_count = part_1(&mut all_foods);
        assert_eq!(5, safe_count);
    }

    #[test]
    fn it_correctly_lists_allergens() {
        let mut all_foods = read_foods(TEST_INPUT.lines());
        let allergens = part_2(&mut all_foods);
        assert_eq!("mxmxvkd,sqjhc,fvjkl", allergens);
    }
}
