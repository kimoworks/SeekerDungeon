using System.Collections.Generic;
using SeekerDungeon.Solana;
using UnityEngine;
using UnityEngine.InputSystem;

namespace SeekerDungeon.Dungeon
{
    /// <summary>
    /// Editor / dev-only component that triggers a fake loot sequence on key press.
    /// Drop on any GameObject in the scene, wire up the references, and press the
    /// trigger key to preview loot animations without any on-chain interaction.
    /// </summary>
    public sealed class LootTestTrigger : MonoBehaviour
    {
        [Header("References")]
        [SerializeField] private LootSequenceController lootSequenceController;
        [SerializeField] private LGGameHudUI gameHudUI;

        [Header("Spawn Position")]
        [Tooltip("World position where the loot items spawn from. If set, uses this transform; otherwise uses this GameObject's position.")]
        [SerializeField] private Transform spawnPoint;

        [Header("Settings")]
        [SerializeField] private Key triggerKey = Key.L;
        [Tooltip("Min number of item stacks per test loot")]
        [SerializeField] private int minItems = 1;
        [Tooltip("Max number of item stacks per test loot")]
        [SerializeField] private int maxItems = 5;
        [Tooltip("Max stack amount per item")]
        [SerializeField] private int maxAmount = 8;

        // All non-legacy item IDs to pick from
        private static readonly ItemId[] AllItems = new[]
        {
            // Weapons
            ItemId.BronzePickaxe, ItemId.IronPickaxe, ItemId.BronzeSword,
            ItemId.IronSword, ItemId.DiamondSword, ItemId.Nokia3310,
            ItemId.WoodenPipe, ItemId.IronScimitar, ItemId.WoodenTankard,
            // Valuables
            ItemId.SilverCoin, ItemId.GoldCoin, ItemId.GoldBar,
            ItemId.Diamond, ItemId.Ruby, ItemId.Sapphire, ItemId.Emerald,
            ItemId.AncientCrown, ItemId.GoblinTooth, ItemId.DragonScale,
            ItemId.CursedAmulet, ItemId.DustyTome, ItemId.EnchantedScroll,
            ItemId.GoldenChalice, ItemId.SkeletonKey, ItemId.MysticOrb,
            ItemId.RustedCompass, ItemId.DwarfBeardRing, ItemId.PhoenixFeather,
            ItemId.VoidShard,
            // Consumables
            ItemId.MinorBuff, ItemId.MajorBuff,
        };

        private void Update()
        {
            var kb = Keyboard.current;
            if (kb == null || !kb[triggerKey].wasPressedThisFrame) return;

            if (lootSequenceController == null)
            {
                Debug.LogWarning("[LootTestTrigger] No LootSequenceController assigned.");
                return;
            }

            if (lootSequenceController.IsPlaying)
            {
                Debug.Log("[LootTestTrigger] Sequence already playing, wait for it to finish.");
                return;
            }

            var loot = GenerateRandomLoot();
            var origin = spawnPoint != null ? spawnPoint.position : transform.position;

            Debug.Log($"[LootTestTrigger] Triggering fake loot with {loot.Items.Count} item(s) at {origin}");

            lootSequenceController.PlayLootSequence(
                loot,
                origin,
                itemId => gameHudUI != null ? gameHudUI.GetSlotScreenPosition(itemId) : null
            );
        }

        private LootResult GenerateRandomLoot()
        {
            var count = Random.Range(minItems, maxItems + 1);
            var items = new List<InventoryItemView>(count);
            var usedIds = new HashSet<ItemId>();

            for (var i = 0; i < count; i++)
            {
                // Pick a random unique item
                ItemId picked;
                var attempts = 0;
                do
                {
                    picked = AllItems[Random.Range(0, AllItems.Length)];
                    attempts++;
                } while (usedIds.Contains(picked) && attempts < 50);

                usedIds.Add(picked);

                // Weapons always amount=1 with durability; valuables/consumables get random stacks
                bool isWeapon = ItemRegistry.IsWearable(picked);
                uint amount = isWeapon ? 1 : (uint)Random.Range(1, maxAmount + 1);
                ushort durability = isWeapon ? (ushort)Random.Range(60, 201) : (ushort)0;

                items.Add(new InventoryItemView
                {
                    ItemId = picked,
                    Amount = amount,
                    Durability = durability,
                });
            }

            return new LootResult { Items = items };
        }
    }
}
