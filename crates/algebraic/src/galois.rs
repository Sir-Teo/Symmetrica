//! Galois Theory Basics
//!
//! This module provides basic Galois group computations for simple field extensions

/// Represents an automorphism of a field extension
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldAutomorphism {
    /// Description of the automorphism
    pub name: String,
    /// Order of the automorphism
    pub order: usize,
}

impl FieldAutomorphism {
    pub fn new(name: String, order: usize) -> Self {
        FieldAutomorphism { name, order }
    }

    /// Identity automorphism
    pub fn identity() -> Self {
        FieldAutomorphism { name: "id".to_string(), order: 1 }
    }
}

/// Represents a Galois group
#[derive(Clone, Debug)]
pub struct GaloisGroup {
    /// Elements of the group (automorphisms)
    pub elements: Vec<FieldAutomorphism>,
    /// Order of the group
    pub order: usize,
    /// Is the extension Galois?
    pub is_galois: bool,
}

impl GaloisGroup {
    pub fn new(elements: Vec<FieldAutomorphism>, is_galois: bool) -> Self {
        let order = elements.len();
        GaloisGroup { elements, order, is_galois }
    }

    /// Trivial Galois group (just identity)
    pub fn trivial() -> Self {
        GaloisGroup { elements: vec![FieldAutomorphism::identity()], order: 1, is_galois: true }
    }

    /// Check if group is cyclic
    pub fn is_cyclic(&self) -> bool {
        // Simplified: groups of prime order are cyclic, and order 1 is trivially cyclic
        self.order == 1 || is_prime(self.order)
    }

    /// Check if group is abelian
    pub fn is_abelian(&self) -> bool {
        // For small groups, most are abelian
        // This is a placeholder - full implementation would check commutativity
        self.order <= 4
    }
}

/// Compute Galois group of Q(√d)/Q
pub fn galois_group_quadratic(_d: i64) -> GaloisGroup {
    // Q(√d)/Q is Galois with Gal(Q(√d)/Q) ≅ Z/2Z
    // Two automorphisms: identity and conjugation (√d ↦ -√d)

    let id = FieldAutomorphism::identity();
    let conj = FieldAutomorphism::new("σ: √d ↦ -√d".to_string(), 2);

    GaloisGroup::new(vec![id, conj], true)
}

/// Compute Galois group of Q(i)/Q where i = √(-1)
pub fn galois_group_gaussian() -> GaloisGroup {
    // Q(i)/Q is Galois with Gal(Q(i)/Q) ≅ Z/2Z
    // Two automorphisms: identity and complex conjugation

    let id = FieldAutomorphism::identity();
    let conj = FieldAutomorphism::new("σ: i ↦ -i".to_string(), 2);

    GaloisGroup::new(vec![id, conj], true)
}

/// Compute Galois group of Q(ζ_n)/Q where ζ_n is primitive nth root of unity
pub fn galois_group_cyclotomic(n: usize) -> GaloisGroup {
    // Gal(Q(ζ_n)/Q) ≅ (Z/nZ)* (units mod n)
    // Order is φ(n) (Euler's totient)

    let phi_n = crate::cyclotomic::euler_phi(n);
    let mut elements = vec![FieldAutomorphism::identity()];

    // Add automorphisms σ_k: ζ_n ↦ ζ_n^k for k coprime to n
    for k in 2..=n {
        if crate::cyclotomic::is_primitive_root(k, n) {
            let name = format!("σ_{}: ζ_{} ↦ ζ_{}^{}", k, n, n, k);
            elements.push(FieldAutomorphism::new(name, phi_n));
        }
    }

    GaloisGroup::new(elements, true)
}

/// Compute Galois group of Q(∛2)/Q
pub fn galois_group_cbrt2() -> GaloisGroup {
    // Q(∛2)/Q is NOT Galois (not a normal extension)
    // The splitting field is Q(∛2, ω) where ω is a primitive cube root of unity
    // Gal(Q(∛2, ω)/Q) ≅ S_3 (symmetric group on 3 elements)

    // For Q(∛2)/Q itself, there's only the identity automorphism
    GaloisGroup::new(vec![FieldAutomorphism::identity()], false)
}

/// Check if extension Q(α)/Q is Galois
/// An extension is Galois if it's both normal and separable
pub fn is_galois_extension(_degree: usize, is_normal: bool, is_separable: bool) -> bool {
    // Over Q (characteristic 0), all extensions are separable
    // So we just need to check normality
    is_normal && is_separable
}

/// Compute the degree of a field extension
pub fn extension_degree(base_degree: usize, extension_degree: usize) -> usize {
    // [K:F] = [K:E][E:F] (tower law)
    base_degree * extension_degree
}

fn is_prime(n: usize) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n.is_multiple_of(2) {
        return false;
    }

    let limit = (n as f64).sqrt() as usize;
    for i in (3..=limit).step_by(2) {
        if n.is_multiple_of(i) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_automorphism() {
        let id = FieldAutomorphism::identity();
        assert_eq!(id.name, "id");
        assert_eq!(id.order, 1);
    }

    #[test]
    fn test_galois_group_quadratic() {
        let g = galois_group_quadratic(2);
        assert_eq!(g.order, 2);
        assert!(g.is_galois);
        assert_eq!(g.elements.len(), 2);
    }

    #[test]
    fn test_galois_group_gaussian() {
        let g = galois_group_gaussian();
        assert_eq!(g.order, 2);
        assert!(g.is_galois);
        assert!(g.is_cyclic());
        assert!(g.is_abelian());
    }

    #[test]
    fn test_galois_group_cyclotomic() {
        // Gal(Q(ζ_3)/Q) has order φ(3) = 2
        let g3 = galois_group_cyclotomic(3);
        assert_eq!(g3.order, 2);
        assert!(g3.is_galois);

        // Gal(Q(ζ_4)/Q) has order φ(4) = 2
        let g4 = galois_group_cyclotomic(4);
        assert_eq!(g4.order, 2);
        assert!(g4.is_galois);

        // Gal(Q(ζ_5)/Q) has order φ(5) = 4
        let g5 = galois_group_cyclotomic(5);
        assert_eq!(g5.order, 4);
        assert!(g5.is_galois);
    }

    #[test]
    fn test_galois_group_cbrt2() {
        let g = galois_group_cbrt2();
        assert_eq!(g.order, 1);
        assert!(!g.is_galois); // Q(∛2)/Q is not Galois
    }

    #[test]
    fn test_is_galois_extension() {
        assert!(is_galois_extension(2, true, true));
        assert!(!is_galois_extension(2, false, true));
        assert!(!is_galois_extension(2, true, false));
    }

    #[test]
    fn test_extension_degree() {
        // [Q(√2, √3) : Q] = [Q(√2, √3) : Q(√2)] * [Q(√2) : Q] = 2 * 2 = 4
        assert_eq!(extension_degree(2, 2), 4);

        // [Q(ζ_5) : Q] = φ(5) = 4
        assert_eq!(extension_degree(1, 4), 4);
    }

    #[test]
    fn test_trivial_galois_group() {
        let g = GaloisGroup::trivial();
        assert_eq!(g.order, 1);
        assert!(g.is_galois);
        assert!(g.is_cyclic());
    }

    #[test]
    fn test_is_prime() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(is_prime(7));
        assert!(!is_prime(9));
        assert!(is_prime(11));
    }
}
