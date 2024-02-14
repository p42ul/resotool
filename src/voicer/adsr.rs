use fundsp::prelude::{clamp01, envelope2, lerp, shared, var, An, EnvelopeIn, Frame, U1};
use fundsp::shared::{Atomic, Shared};
use fundsp::Float;

pub struct Adsr {
    pub attack: Shared<f32>,
    pub decay: Shared<f32>,
    pub sustain: Shared<f32>,
    pub release: Shared<f32>,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            attack: Shared::new(1.0),
            decay: Shared::new(1.0),
            sustain: Shared::new(1.0),
            release: Shared::new(1.0),
        }
    }
}

pub fn adsr_shared<F: Float + Atomic>(
    attack: Shared<F>,
    decay: Shared<F>,
    sustain: Shared<F>,
    release: Shared<F>,
) -> An<EnvelopeIn<F, F, impl Fn(F, &Frame<F, U1>) -> F + Sized + Clone, U1, F>> {
    let neg1 = F::from_f64(-1.0);
    let zero = F::from_f64(0.0);
    let a = shared(neg1);
    let b = shared(neg1);
    let attack_start = var(&a);
    let release_start = var(&b);
    envelope2(move |time, control| {
        if attack_start.value() < zero && control > zero {
            attack_start.set_value(time);
            release_start.set_value(neg1);
        } else if release_start.value() < zero && control <= zero {
            release_start.set_value(time);
            attack_start.set_value(neg1);
        }
        clamp01(if release_start.value() < zero {
            ads(
                attack.clone(),
                decay.clone(),
                sustain.clone(),
                time - attack_start.value(),
            )
        } else {
            releasing(
                sustain.clone(),
                release.clone(),
                time - release_start.value(),
            )
        })
    })
}

fn ads<F: Float + Atomic>(attack: Shared<F>, decay: Shared<F>, sustain: Shared<F>, time: F) -> F {
    if time < attack.value() {
        lerp(F::from_f64(0.0), F::from_f64(1.0), time / attack.value())
    } else {
        let decay_time = time - attack.value();
        if decay_time < decay.value() {
            lerp(
                F::from_f64(1.0),
                sustain.value(),
                decay_time / decay.value(),
            )
        } else {
            sustain.value()
        }
    }
}

fn releasing<F: Float + Atomic>(sustain: Shared<F>, release: Shared<F>, release_time: F) -> F {
    if release_time > release.value() {
        F::from_f64(0.0)
    } else {
        lerp(
            sustain.value(),
            F::from_f64(0.0),
            release_time / release.value(),
        )
    }
}
