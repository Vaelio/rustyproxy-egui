#[derive(Debug)]
pub enum FilterCat{
    Host,
    Code,
    Source,
    Path,
}


impl FilterCat {
    pub fn from_filtercat(f: &FilterCat) -> FilterCat {
        match f {
            FilterCat::Host => FilterCat::Host,
            FilterCat::Code => FilterCat::Code,
            FilterCat::Source => FilterCat::Source,
            FilterCat::Path => FilterCat::Path,
        }
    }

    pub fn from_filtercat_opt(o: &Option<FilterCat>) -> Option<FilterCat> {
        match o {
            None => None,
            Some(f) => Some(Self::from_filtercat(&f)),
        }
    }
}