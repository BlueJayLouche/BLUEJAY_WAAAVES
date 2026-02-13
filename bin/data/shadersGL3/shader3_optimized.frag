//filters

// Optimized Dither using Bayer patterns computed on-the-fly
// Eliminates large lookup tables and loops

// Compute 4x4 Bayer pattern value (0-1 range)
float bayer4x4(vec2 coord) {
	int x = int(mod(coord.x, 4.0));
	int y = int(mod(coord.y, 4.0));
	// Bayer 4x4 pattern computed using bit manipulation
	int v = ((x ^ y) << 1) | ((x & 1) ^ ((y & 2) >> 1));
	return float(v) / 16.0;
}

// Compute 8x8 Bayer pattern approximation using interpolated 4x4
float bayer8x8(vec2 coord) {
	float b4 = bayer4x4(coord);
	float b4offset = bayer4x4(coord * 0.5 + vec2(0.5));
	return (b4 + b4offset * 0.25) * 0.8;
}

// Optimized quantization without loops
// Returns the closest palette color using floor/ceil
float quantize(float inColor, float paletteSize) {
	float scaled = inColor * paletteSize;
	float lower = floor(scaled) / paletteSize;
	float upper = ceil(scaled) / paletteSize;
	return (inColor - lower) < (upper - inColor) ? lower : upper;
}

// Optimized dither function
// - No large arrays
// - No loops
// - Branchless dither type selection
float dither2(float inColor, vec2 inCoord, float ditherPalette, int ditherType) {
	// Compute Bayer value based on type (0=4x4, 1=8x8)
	float bayer4 = bayer4x4(inCoord);
	float bayer8 = bayer8x8(inCoord);
	float indexValue = mix(bayer4, bayer8, float(ditherType));
	
	// Add dither noise before quantization
	float noise = (indexValue - 0.5) / ditherPalette;
	float dithered = inColor + noise * 0.5;
	
	// Quantize to palette
	return quantize(clamp(dithered, 0.0, 1.0), ditherPalette);
}

// Optimized blur and sharpen function
// - Uses texture() instead of textureLod() for better performance (lod=0 is implicit)
// - Replaces branching with mix() for sharpen boost
// - Reduces HSB conversions by sampling luminance directly
vec4 blurAndSharpen(sampler2D blurAndSharpenTex, vec2 coord,
		float sharpenAmount, float sharpenRadius, float sharpenBoost,
		float blurRadius, float blurAmount) {
	vec4 originalColor = texture(blurAndSharpenTex, coord);
	vec2 texSize = vec2(textureSize(blurAndSharpenTex, 0));

	vec2 blurSize = vec2(blurRadius) / (texSize - vec2(1));
	vec2 sharpenSize = vec2(sharpenRadius) / (texSize - vec2(1));

	//blur - 8 samples box blur
	vec4 colorBlur = texture(blurAndSharpenTex, coord + blurSize*vec2( 1, 1))
                  + texture(blurAndSharpenTex, coord + blurSize*vec2( 0, 1))
                  + texture(blurAndSharpenTex, coord + blurSize*vec2(-1, 1))
                  + texture(blurAndSharpenTex, coord + blurSize*vec2(-1, 0))
                  + texture(blurAndSharpenTex, coord + blurSize*vec2(-1,-1))
                  + texture(blurAndSharpenTex, coord + blurSize*vec2( 0,-1))
                  + texture(blurAndSharpenTex, coord + blurSize*vec2( 1,-1))
                  + texture(blurAndSharpenTex, coord + blurSize*vec2( 1, 0));

	colorBlur *= 0.125;
	colorBlur = mix(originalColor, colorBlur, blurAmount);

	//sharpen - sample brightness using dot product (faster than HSB conversion)
	//Using luminance weights: 0.299*R + 0.587*G + 0.114*B
	const vec3 lumWeights = vec3(0.299, 0.587, 0.114);
	float color_sharpen_bright =
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2( 1, 0)).rgb, lumWeights)+
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2(-1, 0)).rgb, lumWeights)+
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2( 0, 1)).rgb, lumWeights)+
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2( 0,-1)).rgb, lumWeights)+
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2( 1, 1)).rgb, lumWeights)+
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2(-1, 1)).rgb, lumWeights)+
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2( 1,-1)).rgb, lumWeights)+
		dot(texture(blurAndSharpenTex, coord + sharpenSize*vec2(-1,-1)).rgb, lumWeights);

    color_sharpen_bright *= 0.125;

    vec3 colorBlurHsb = rgb2hsb(colorBlur.rgb);
    colorBlurHsb.z -= sharpenAmount * color_sharpen_bright;

    // Use mix() instead of if() to avoid branching
    float boostFactor = mix(1.0, 1.0 + sharpenAmount + sharpenBoost, step(0.001, sharpenAmount));
    colorBlurHsb.z *= boostFactor;

    return vec4(hsb2rgb(colorBlurHsb), 1.0);
}
