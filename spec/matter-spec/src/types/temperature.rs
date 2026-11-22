//! Matter `temperature` semantic type (hundredths of a degree Celsius).

use core::fmt;

#[cfg(feature = "uom")]
use uom::si::{f32::ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

super::define_matter_scalar_type! {
    /// Temperature in hundredths of a degree Celsius (`0.01 °C` units, wire type `i16`).
    ///
    /// Construct explicitly with [`Temperature::from_raw`] or [`Temperature::from_celsius`] /
    /// [`Temperature::from_fahrenheit`] (no `From<i16>` is implemented).
    ///
    /// ```rust
    /// use matter_spec::types::Temperature;
    ///
    /// let t = Temperature::from_celsius(21.5);
    /// assert_eq!(t.raw(), 2150);
    /// assert!((t.to_fahrenheit() - 70.7).abs() < 0.1);
    /// ```
    pub struct Temperature(i16);
    attr_variant = I16;
}

impl Temperature {
    /// Maximum valid Matter temperature (`327.67 °C`).
    pub const MAX: Self = Self(i16::MAX);
    /// Minimum valid Matter temperature (`-273.15 °C`, absolute zero).
    pub const MIN: Self = Self(-27_315);

    /// Construct from degrees Celsius, rounding to the nearest `0.01 °C` and clamping to [`Self::MIN`]..=[`Self::MAX`].
    #[inline]
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "no_std float scaling"
    )]
    #[must_use]
    pub fn from_celsius(celsius: f32) -> Self {
        if celsius.is_nan() {
            return Self(0);
        }
        let scaled = celsius * 100.0;
        let rounded = if scaled >= 0.0 { scaled + 0.5 } else { scaled - 0.5 };
        let clamped = rounded.clamp(-27_315.0, 32_767.0);
        Self(clamped as i16)
    }

    /// Convert to degrees Celsius.
    #[inline]
    #[must_use]
    pub fn to_celsius(self) -> f32 {
        f32::from(self.0) / 100.0
    }

    /// Construct from degrees Fahrenheit, rounding to the nearest `0.01 °C`.
    #[inline]
    #[must_use]
    pub fn from_fahrenheit(fahrenheit: f32) -> Self {
        Self::from_celsius((fahrenheit - 32.0) * (5.0 / 9.0))
    }

    /// Convert to degrees Fahrenheit.
    #[inline]
    #[must_use]
    pub fn to_fahrenheit(self) -> f32 {
        self.to_celsius() * (9.0 / 5.0) + 32.0
    }
}

super::impl_matter_uom_unit!(
    Temperature,
    ThermodynamicTemperature,
    degree_celsius,
    from_celsius,
    to_celsius
);

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}°C", self.to_celsius())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AttributeValue, MatterStatus};

    #[test]
    fn celsius_round_trip() {
        let t = Temperature::from_celsius(21.5);
        assert_eq!(t.raw(), 2150);
        assert!((t.to_celsius() - 21.5).abs() < 0.01);

        let neg = Temperature::from_celsius(-10.25);
        assert_eq!(neg.raw(), -1025);
        assert!((neg.to_celsius() - (-10.25)).abs() < 0.01);

        let zero = Temperature::from_celsius(0.0);
        assert_eq!(zero.raw(), 0);
    }

    #[test]
    fn fahrenheit_round_trip() {
        let t = Temperature::from_fahrenheit(68.0);
        assert_eq!(t.raw(), 2000);
        assert!((t.to_fahrenheit() - 68.0).abs() < 0.05);
    }

    #[test]
    fn attribute_value_round_trip() {
        use crate::model::Nullable;

        let t = Temperature::from_raw(2345);
        let av = AttributeValue::from(t);
        assert_eq!(av, AttributeValue::I16(2345));
        assert_eq!(Temperature::try_from(&av), Ok(t));
        assert_eq!(Temperature::try_from(av.clone()), Ok(t));
        assert_eq!(Nullable::<Temperature>::try_from(av), Ok(Nullable::NonNull(t)));
        assert_eq!(
            Temperature::try_from(&AttributeValue::U16(2345)),
            Err(MatterStatus::InvalidDataType)
        );
    }

    #[test]
    fn temperature_clamping_and_nan() {
        assert_eq!(Temperature::from_celsius(-500.0), Temperature::MIN);
        assert_eq!(Temperature::from_celsius(-500.0).raw(), -27_315);
        assert_ne!(Temperature::from_celsius(-500.0).raw(), i16::MIN);
        assert_eq!(Temperature::from_celsius(f32::NEG_INFINITY), Temperature::MIN);
        assert_eq!(Temperature::from_celsius(500.0), Temperature::MAX);
        assert_eq!(Temperature::from_celsius(500.0).raw(), i16::MAX);
        assert_eq!(Temperature::from_celsius(f32::INFINITY), Temperature::MAX);
        assert_eq!(Temperature::from_celsius(f32::NAN), Temperature::from_raw(0));
    }

    #[cfg(feature = "uom")]
    #[test]
    fn uom_round_trip() {
        let t = Temperature::from_celsius(21.5);
        let u = ThermodynamicTemperature::from(t);
        assert_eq!(Temperature::from(u), t);
    }
}
