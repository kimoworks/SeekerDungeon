using UnityEngine;

namespace SeekerDungeon.Solana
{
    public enum SolanaRuntimeNetwork
    {
        Devnet = 0,
        Mainnet = 1
    }

    public enum SkrMintMode
    {
        DevnetMock = 0,
        MainnetReal = 1,
        Custom = 2
    }

    [CreateAssetMenu(
        fileName = "LGSolanaDeploymentConfig",
        menuName = "SeekerDungeon/Solana/Deployment Config")]
    public sealed class LGSolanaDeploymentConfig : ScriptableObject
    {
        [Header("Network")]
        [SerializeField] private SolanaRuntimeNetwork solanaNetwork = SolanaRuntimeNetwork.Devnet;
        [SerializeField] private string devnetRpcUrl = LGConfig.RPC_URL;
        [SerializeField] private string devnetFallbackRpcUrl = LGConfig.RPC_FALLBACK_URL;
        [SerializeField] private string mainnetRpcUrl = LGConfig.MAINNET_RPC_URL;
        [SerializeField] private string mainnetFallbackRpcUrl = LGConfig.MAINNET_RPC_FALLBACK_URL;

        [Header("SKR Mint")]
        [SerializeField] private SkrMintMode skrMintMode = SkrMintMode.DevnetMock;
        [SerializeField] private string customSkrMint = string.Empty;

        public SolanaRuntimeNetwork SolanaNetwork => solanaNetwork;
        public string DevnetRpcUrl => devnetRpcUrl;
        public string DevnetFallbackRpcUrl => devnetFallbackRpcUrl;
        public string MainnetRpcUrl => mainnetRpcUrl;
        public string MainnetFallbackRpcUrl => mainnetFallbackRpcUrl;
        public SkrMintMode SkrMintMode => skrMintMode;
        public string CustomSkrMint => customSkrMint;
    }
}
