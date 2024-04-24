#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(test)]
mod tests { 
    use traitenum::enumtrait;
    
    #[test]
    fn attribute_model() {
        #[enumtrait()]
        pub trait MyTrait {
            // test default parsing
            fn name(&self) -> &'static str;
            // test ordinal
            #[enumtrait::Num(default(42))]
            fn column(&self) -> usize;
            // test default implementation
            fn something_default(&self) {
                todo!();
            }
        }
    }

    #[test]
    fn sample_usage() {
        //#derive(traitenum::TraitEnum)
        //#traitenum(trait(CuisineTrait))
        //#traitenum(relation(FastFood), many)
        enum Cuisine {
            American,
            Canadian,
            Mexican,
        }

        //traitenum! FastFoodTrait {
        //   name, id, calories, price,
        //   ordinal: ordinal,
        //}

        //OR

        //#[derive(traitenum::TraitEnum)]
        //#[traitenum(trait(FastFoodTrait))]  <-- imports from file via serde. by type id?

        //OR

        //#[derive(traitenum::TraitEnum)]
        //#[traitenum(trait(FastFoodTrait))]
        //#[traitenum(method(ordinal), ordinal)]
        //#[traitenum(method(name), method(id), method(calories), method(price)]
        //#[traitenum(method(column), serial, start(1), increment(1))]
        //#[traitenum(method(rating), default(0)]
        //#[traitenum(method(prompt), format("Would you like some {}?", name))]
        //#[traitenum(method(cuisine), relation(CuisineTrait), one)]
         enum AmericanFastFood {
           //#[traitenum(cuisine(Cuisine::American), name("Fried Chicken"), id(55), rating(0), price(9), calories(550))]
            FriedChicken,
            Hamburger,
            //#[traitenum(cuisine(Cuisine::American), name("Hotdog"), id(4), rating(-2), price(3.25), calories(800))]
            Hotdog,
        }

        enum CanadianFastFood {
            Donair,
            Poutine
        }

        enum MexicanFastFood {
            //#[traitenum(cuisine(Cuisine::Mexican), name("Burrito"), id(321), rating(4), price(6.50), calories(600))]
            Burrito,
            Nachos,
            Taco,
        }

        //#derive(traitenum::EnumTrait)  <-- automatically exports via serde (and typeid?)

        //OR

        //#[traitenum(method(column), serial, start(1), increment(1))]
        //#[traitenum(method(prompt), format("Would you like some {}?", name))]
        //#[traitenum(method(ordinal), ordinal)]
        //#[traitenum(method(cuisine), relation(Cuisine), one)]

        /*traitenum::enumtrait!{
            name: str { key: value },
            column: serial { start: 1, increment: 1 },
            prompt: relation { enum: Cuisine, type: one-to-many }
        };*/

        trait FastFoodTrait {
            fn ordinal(&self) -> usize;
            fn name(&self) -> &'static str;
            fn cuisine(&self) -> Cuisine;
            fn id(&self) -> u64;
            fn column(&self) -> u8;
            fn rating(&self) -> i8;
            fn price(&self) -> f32;
            fn prompt(&self) -> &'static str;
            fn calories(&self) -> u64;
        }

        //assert_eq!("Fried Chicken", FastFood::FriedChicken::name())
        //assert_eq!(Cuisine::Mexican, FastFood::Taco::cuisine())
    }
}
