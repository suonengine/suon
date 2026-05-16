# suon_vitals

Shared Bevy components for gameplay vitals.

`suon_vitals` provides:

- `VitalsPlugins` - observer group that applies vital intents and clamps current values
- `Health`, `MaxHealth` - current and maximum hit points
- `HealIntent`, `DamageIntent`, `Heal`, `Damage`, `HealRejected`, `DamageRejected` - health update flow
- `Die`, `Revive` - death state transition events
- `FleeHealth`, `EnterFleeHealth`, `ExitFleeHealth` - monster flee threshold flow
- `Mana`, `MaxMana` - current and maximum mana points
- `RestoreManaIntent`, `ConsumeManaIntent`, `RestoreMana`, `ConsumeMana`, `ManaDepleted`, `ManaRecovered`, `RestoreManaRejected`, `ConsumeManaRejected` - mana update flow
- `ManaShield`, `MaxManaShield` - temporary mana shield bar values
- `RestoreManaShieldIntent`, `AbsorbManaShieldIntent`, `RestoreManaShield`, `AbsorbManaShield`, `ManaShieldBroken`, `ManaShieldActivated`, `RestoreManaShieldRejected`, `AbsorbManaShieldRejected` - mana shield update flow
- `Soul`, `MaxSoul` - current and maximum soul points
- `RestoreSoulIntent`, `ConsumeSoulIntent`, `RestoreSoul`, `ConsumeSoul`, `SoulDepleted`, `SoulRecovered`, `RestoreSoulRejected`, `ConsumeSoulRejected` - soul update flow
- `Stamina`, `MaxStamina` - stamina duration
- `RestoreStaminaIntent`, `ConsumeStaminaIntent`, `RestoreStamina`, `ConsumeStamina`, `StaminaExhausted`, `StaminaRecovered`, `RestoreStaminaRejected`, `ConsumeStaminaRejected` - stamina update flow
- `BaseSpeed`, `SpeedModifiers`, `Speed`, `SetBaseSpeedIntent`, `AddSpeedModifierIntent`, `RemoveSpeedModifierIntent`, `SpeedChanged`, `SetBaseSpeedRejected`, `AddSpeedModifierRejected`, `RemoveSpeedModifierRejected` - movement speed flow
- `Capacity`, `CapacityModifiers`, `MaxCapacity`, `FreeCapacity` - base, modifiers, effective maximum and remaining carrying capacity
- `RestoreCapacityIntent`, `ConsumeCapacityIntent`, `RestoreCapacity`, `ConsumeCapacity`, `CapacityFull`, `CapacityAvailable`, `RestoreCapacityRejected`, `ConsumeCapacityRejected` - free capacity update flow
- `Level`, `Experience`, `MagicLevel`, `ManaSpent` - player progression values tied to vitals
- `HealthGain`, `ManaGain`, `SoulGain` and tick components - regeneration settings
- `HealthCost`, `ManaCost`, `SoulCost` and percent cost components - spell and weapon costs

## Installation

```toml
[dependencies]
suon_vitals = { path = "../suon_vitals" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_vitals::prelude::*;

let mut app = App::new();
app.add_plugins(MinimalPlugins);
app.add_plugins(VitalsPlugins);

let entity = app
    .world_mut()
    .spawn((
        MaxHealth(150),
        MaxMana(40),
        MaxSoul(100),
        MaxStamina::from_minutes(2520),
        Capacity(400),
    ))
    .id();

app.world_mut().trigger(HealIntent {
    entity,
    amount: 150,
});
app.update();

assert_eq!(
    **app
        .world()
        .get::<Health>(entity)
        .expect("Health should exist"),
    150
);
```

## Synchronization

`VitalsPlugins` registers observers that apply intent events and keep current values within their maxima:

- `Health <= MaxHealth`
- `Mana <= MaxMana`
- `Soul <= MaxSoul`
- `Stamina <= MaxStamina`
- `ManaShield <= MaxManaShield`

Use intents for gameplay changes:

```rust,ignore
app.world_mut().trigger(ConsumeManaIntent {
    entity,
    amount: 20,
});
```

Intent observers reject no-op boundary requests:

- empty amounts
- damage when health is already zero
- consume when mana, soul, stamina, mana shield or free capacity is already zero
- restore when the value is already at its maximum

Derived values that need outside context, such as free capacity, magic level progression,
regeneration ticks and protocol serialization, stay in higher-level gameplay crates.
