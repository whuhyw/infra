using System.Text;
using Org.BouncyCastle.Asn1.GM;
using Org.BouncyCastle.Crypto;
using Org.BouncyCastle.Crypto.Engines;
using Org.BouncyCastle.Crypto.Parameters;
using Org.BouncyCastle.Security;
using Org.BouncyCastle.Utilities.Encoders;

namespace InformationProvider.Services;

public class Sm2CryptoService : ISm2CryptoService
{
    private readonly ECPublicKeyParameters _publicKey;

    public Sm2CryptoService(string publicKeyHex)
    {
        var domain = GMNamedCurves.GetByName("sm2p256v1")
            ?? throw new InvalidOperationException("SM2 curve not found");
        var point = domain.Curve.DecodePoint(Hex.Decode(publicKeyHex));
        _publicKey = new ECPublicKeyParameters(
            point, new ECDomainParameters(domain));
    }

    public string Encrypt(string plaintext)
    {
        var engine = new SM2Engine(SM2Engine.Mode.C1C3C2);
        var random = new SecureRandom();
        engine.Init(true, new ParametersWithRandom(_publicKey, random));

        var input = Encoding.UTF8.GetBytes(plaintext);
        var ciphertext = engine.ProcessBlock(input, 0, input.Length);

        return Convert.ToHexString(ciphertext).ToLower();
    }
}
