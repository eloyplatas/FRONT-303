#ifndef SHARANDOM
#define SHARANDOM

#include <vector>

namespace AudioLib
{
	class ShaRandom
	{
	public:
		static std::vector<double> Generate(long long seed, int count);
		static std::vector<double> Generate(long long seed, int count, double crossSeed);
		// Front 303: the same series written into `out` (at most 128 values) without
		// touching the heap, so parameter changes are safe on the audio thread
		static void GenerateInto(long long seed, int count, double crossSeed, double* out);
	};
}

#endif
