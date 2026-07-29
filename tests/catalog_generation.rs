//! Public catalog-generation admission and transition laws.

use keep::{CatalogGeneration, CatalogGenerationError};

#[test]
fn catalog_generation_admits_only_positive_values() -> Result<(), CatalogGenerationError> {
    assert!(matches!(
        CatalogGeneration::new(0),
        Err(CatalogGenerationError::Zero)
    ));

    let first = CatalogGeneration::new(1)?;
    assert_eq!(first.get(), 1);
    Ok(())
}

#[test]
fn catalog_generation_successor_is_checked() -> Result<(), CatalogGenerationError> {
    let first = CatalogGeneration::new(1)?;
    let second = first.successor()?;
    assert_eq!(second.get(), 2);

    let maximum = CatalogGeneration::new(u64::MAX)?;
    assert!(matches!(
        maximum.successor(),
        Err(CatalogGenerationError::Exhausted { current: u64::MAX })
    ));
    Ok(())
}

#[test]
fn catalog_generation_order_is_canonical() -> Result<(), CatalogGenerationError> {
    let first = CatalogGeneration::new(1)?;
    let second = CatalogGeneration::new(2)?;

    assert!(first < second);
    assert_eq!([second, first].into_iter().min(), Some(first));
    Ok(())
}
