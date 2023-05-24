use entities::power_interruptions::location::AreaName;
use itertools::Itertools;
use std::fmt::format;

struct SearcheableCandidateInner(Vec<String>);

impl SearcheableCandidateInner {
    fn new(candidate: String) -> Self {
        let split = candidate.split_once("&");
        let mut result = vec![];
        if let Some((before, after)) = split {
            let before = before.trim();
            let after = after.trim();
            result.extend(Self::split_1(before, after));
            result.extend(Self::split_2(before, after));
            result.extend(Self::split_3(before, after));
            result.extend(Self::split_4(before, after));
            result.extend(Self::split_5(before, after));
            result.extend(Self::split_6(before, after))
        }

        let mut unique_splits = result
            .into_iter()
            .map(|data| data.trim().split(" ").unique().join(" "))
            .unique()
            .collect_vec();

        // let mut substring_indeces_to_remove = Vec::new();
        // println!("unique splits -> {unique_splits:?}");
        // for (index, outer_string) in unique_splits.iter().enumerate() {
        //     for (index2, inner_string) in unique_splits.iter().enumerate() {
        //         if index == index2 {
        //             continue;
        //         }
        //         println!(
        //             "outer_string: {}, inner_string: {} at index {index}",
        //             outer_string, inner_string
        //         );
        //         if outer_string.contains(inner_string) {
        //             substring_indeces_to_remove.push(index2);
        //         }
        //     }
        // }
        //
        // substring_indeces_to_remove.sort_unstable();
        // substring_indeces_to_remove
        //     .into_iter()
        //     .rev()
        //     .for_each(|index| {
        //         unique_splits.remove(index);
        //     });

        SearcheableCandidateInner(unique_splits)
    }

    fn into_inner(self) -> Vec<String> {
        self.0
    }

    // Split Shell & Total Petro Stns Kiambu Road to vec![Shell Petro Stns Kiambu Road, Total Petro Stns Kiambu Road]
    fn split_1(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let after_as_list = after_amperstand.split(" ").collect_vec();
        // If after_amperstand is only one word, then this split method isnt applicable.
        // eg: Makueni boys & girls
        if after_as_list.len() == 1 {
            return Default::default();
        }
        let data = after_as_list.split_first();
        if let Some((first, rest)) = data {
            let rest_as_str = rest.join(" ");
            let result = vec![
                format!("{} {}", before_amperstand, &rest_as_str),
                format!("{} {}", &first, &rest_as_str),
            ];
            println!("split_1: {:?}", result);
            result
        } else {
            vec![]
        }
    }

    // Split Kawangware DC & DO Offices to vec![Kawangware DC Offices, Kawangware DO Offices]
    fn split_2(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let mut result = vec![];
        let before_as_list = before_amperstand.split(" ").collect_vec();
        // If before_amperstand is only one word, then this split method isnt applicable.
        if before_as_list.len() == 1 {
            return Default::default();
        }
        if let Some(first) = before_as_list.first() {
            result.push(format!("{} {}", first, after_amperstand));
        }

        let after_as_list = after_amperstand.split(" ").collect_vec();
        // If after_amperstand is only one word, then this split method isnt applicable.
        if after_as_list.len() == 1 {
            return Default::default();
        }
        let last_and_elements_after = after_as_list.split_last();
        if let Some((_last, rest)) = last_and_elements_after {
            let rest_as_str = rest.join(" ");
            result.push(format!("{} {}", before_amperstand, rest_as_str));
        }
        println!("split_2 result: {:?}", result);
        result
    }

    // Split Makueni Boys & Girls to vec![Makueni Boys, Makueni Girls]
    fn split_3(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        // If before_amperstand is only one word, then this split method isnt applicable.
        // eg: 2nd & 3rd Parklands, we will end up with "2nd" as a candidate, which is too general.
        if before_amperstand.split(" ").count() == 1 {
            return Default::default();
        }
        let mut result = vec![before_amperstand.to_string()];

        let before_as_list = before_amperstand.split(" ").collect_vec();
        if let Some(first) = before_as_list.first() {
            result.push(format!("{} {}", first, after_amperstand));
        }

        println!("split_3 result: {:?}", result);
        result
    }

    // Split Warai South & Warai North Road to vec![Warai South Road, Warai North Road]
    fn split_4(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let mut result = vec![after_amperstand.to_string()];
        let after_as_list = after_amperstand.split(" ").collect_vec();

        // If after_amperstand is only one word, then this split method isnt applicable.
        // eg: Makueni Boys & Girls, we will end up with "Girls" as a candidate, which is too general.
        if after_as_list.len() == 1 || before_amperstand.split(" ").count() == 1 {
            return Default::default();
        }
        if let Some(last) = after_as_list.last() {
            result.push(format!("{} {}", before_amperstand, last));
        }

        println!("split_4 result: {:?}", result);
        result
    }

    // Split St Lwanga Catholic Church & School to vec![St Lwanga Catholic Church, St Lwanga Catholic School]
    fn split_5(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let before_as_list = before_amperstand.split(" ").collect_vec();
        // If before_amperstand is only one word, then this split method isnt applicable.
        // eg: 2nd & 3rd Parklands, we will end up with "2nd" as a candidate, which is too general.
        if before_as_list.len() == 1 {
            return Default::default();
        }
        let last_and_elements_before = before_as_list.split_last();
        if let Some((_last, elements_before_last_word)) = last_and_elements_before {
            let elements_before_last_word = elements_before_last_word.join(" ");
            let final_word = format!("{} {}", elements_before_last_word, after_amperstand);
            let result = vec![format!("{}", before_amperstand), final_word];

            println!("split_5 result: {:?}", result);
            result
        } else {
            vec![]
        }
    }

    // Split GSU & AP to vec![GSU, AP]
    fn split_6(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let phrase_without_amperstand = format!("{} {}", before_amperstand, after_amperstand);
        // only return result, if phrase_without_amperstand is two words.
        if phrase_without_amperstand.split(" ").count() == 2 {
            let result = vec![before_amperstand.to_string(), after_amperstand.to_string()];

            println!("split_6 result: {:?}", result);

            return result;
        }

        Default::default()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SearcheableCandidates(String);

impl ToString for SearcheableCandidates {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl AsRef<str> for SearcheableCandidates {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SearcheableCandidates {
    pub fn from_area_name(area: &AreaName) -> Vec<Self> {
        area.as_ref()
            .split(',')
            .map(SearcheableCandidates::from)
            .collect_vec()
    }

    // pub fn original_value(&self) -> String {
    //     self.0.replace(" <-> ", " ")
    // }
}

impl From<&str> for SearcheableCandidates {
    fn from(value: &str) -> Self {
        let value = value.trim().replace(' ', " <-> ");
        SearcheableCandidates(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableCandidateInner;
    use itertools::Itertools;
    use rstest::rstest;
    use std::collections::HashSet;

    #[rstest]
    #[case("2nd & 3rd Parklands", vec!["2nd Parklands", "3rd Parklands"])]
    #[case("GSU & AP", vec!["GSU", "AP"])]
    #[case("Makueni Boys & Girls", vec!["Makueni Boys", "Makueni Girls"])]
    #[case("Kabare Market & Girls High School", vec!["Kabare Market", "Kabare Girls High School"])]
    #[case("Kimunye T /Fact & Market", vec!["Kimunye T /Fact", "Kimunye Market"])]
    #[case("St Lwanga Catholic Church & School", vec!["St Lwanga Catholic Church", "St Lwanga Catholic School"])]
    #[case("Shell & Total Petro Stns Kiambu Road", vec!["Shell Petro Stns Kiambu Road", "Total Petro Stns Kiambu Road"])]
    #[case("Warai South & Warai North Road", vec!["Warai South Road", "Warai North Road"])]
    #[case("Kawangware DC & DO Offices", vec!["Kawangware DC Offices", "Kawangware DO Offices"])]
    fn test_searcheable_candidate_inner(#[case] input: &str, #[case] expected: Vec<&str>) {
        let candidate = SearcheableCandidateInner::new(input.to_string());
        let result = candidate.into_inner().into_iter().collect::<HashSet<_>>();
        let expected_as_hash_set = expected
            .into_iter()
            .map(|data| data.to_owned())
            .collect::<HashSet<_>>();
        println!("{result:?}");
        assert_eq!(expected_as_hash_set.difference(&result).count(), 0)
    }
}
