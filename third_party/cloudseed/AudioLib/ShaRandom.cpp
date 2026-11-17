
#include <climits>
#include "ShaRandom.h"
#include "../Utils/Sha256.h"
#include <cstring>

namespace AudioLib
{
	using namespace std;

	vector<double> ShaRandom::Generate(long long seed, int count)
	{
		vector<unsigned char> byteList;
		auto iterations = count * sizeof(unsigned int) / (256 / 8) + 1;
		auto byteArr = (unsigned char*)&seed;
		vector<unsigned char> bytes(byteArr, byteArr + 8);

		for (size_t i = 0; i < iterations; i++)
		{
			bytes = sha256(&bytes[0], 8);
			for (auto b : bytes)
				byteList.push_back(b);
		}

		auto intArray = (unsigned int*)(&byteList[0]);
		vector<double> output;

		for (int i = 0; i < count; i++)
		{
			unsigned int val = intArray[i];
			double doubleVal = val / (double)UINT_MAX;
			output.push_back(doubleVal);
		}

		return output;
	}

	vector<double> ShaRandom::Generate(long long seed, int count, double crossSeed)
	{
		auto seedA = seed;
		auto seedB = ~seed;
		auto seriesA = Generate(seedA, count);
		auto seriesB = Generate(seedB, count);

		vector<double> output;
		for (int i = 0; i < count; i++)
			output.push_back(seriesA[i] * (1 - crossSeed) + seriesB[i] * crossSeed);

		return output;
	}

	static void seriesInto(long long seed, int count, double* out)
	{
		unsigned char bytes[SHA256::DIGEST_SIZE] = {};
		unsigned char words[SHA256::DIGEST_SIZE * 17] = {};
		memcpy(bytes, &seed, 8);
		auto iterations = count * sizeof(unsigned int) / (256 / 8) + 1;
		for (size_t i = 0; i < iterations && i < 17; i++)
		{
			SHA256 ctx;
			ctx.init();
			ctx.update(bytes, 8);
			ctx.final(bytes);
			memcpy(words + i * SHA256::DIGEST_SIZE, bytes, SHA256::DIGEST_SIZE);
		}
		for (int i = 0; i < count; i++)
		{
			unsigned int val;
			memcpy(&val, words + i * sizeof(unsigned int), sizeof(unsigned int));
			out[i] = val / (double)UINT_MAX;
		}
	}

	void ShaRandom::GenerateInto(long long seed, int count, double crossSeed, double* out)
	{
		double a[128], b[128];
		if (count > 128) count = 128;
		seriesInto(seed, count, a);
		seriesInto(~seed, count, b);
		for (int i = 0; i < count; i++)
			out[i] = a[i] * (1 - crossSeed) + b[i] * crossSeed;
	}
}
