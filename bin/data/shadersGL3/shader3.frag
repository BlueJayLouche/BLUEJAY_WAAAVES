OF_GLSL_SHADER_HEADER

const float PI=3.1415926535;
const float TWO_PI=6.2831855;


uniform sampler2D block2Output;
uniform sampler2D block1Output;

uniform float ratio;

uniform float width;
uniform float height;
uniform float inverseWidth;
uniform float inverseHeight;


//block1 geo
uniform vec2 block1XYDisplace;
uniform float block1ZDisplace;
uniform float block1Rotate;
uniform vec4 block1ShearMatrix;
uniform float block1KaleidoscopeAmount;
uniform float block1KaleidoscopeSlice;
uniform int block1HMirror;
uniform int block1VMirror;
uniform int	block1RotateMode;
uniform	int block1GeoOverflow;
uniform int block1HFlip;
uniform int block1VFlip;

//block1 colorize
uniform int block1ColorizeSwitch;
uniform int block1ColorizeHSB_RGB;
uniform vec3 block1ColorizeBand1;
uniform vec3 block1ColorizeBand2;
uniform vec3 block1ColorizeBand3;
uniform vec3 block1ColorizeBand4;
uniform vec3 block1ColorizeBand5;

//block1 filters
uniform	float block1BlurAmount;
uniform	float block1BlurRadius;
uniform	float block1SharpenAmount;
uniform	float block1SharpenRadius;
uniform float block1FiltersBoost;
uniform float block1Dither;
uniform int block1DitherSwitch;
uniform int block1DitherType;

//block2 geo
uniform vec2 block2XYDisplace;
uniform float block2ZDisplace;
uniform float block2Rotate;
uniform vec4 block2ShearMatrix;
uniform float block2KaleidoscopeAmount;
uniform float block2KaleidoscopeSlice;
uniform int block2HMirror;
uniform int block2VMirror;
uniform int	block2RotateMode;
uniform	int block2GeoOverflow;
uniform int block2HFlip;
uniform int block2VFlip;

//block2 colorize
uniform int block2ColorizeSwitch;
uniform int block2ColorizeHSB_RGB;
uniform vec3 block2ColorizeBand1;
uniform vec3 block2ColorizeBand2;
uniform vec3 block2ColorizeBand3;
uniform vec3 block2ColorizeBand4;
uniform vec3 block2ColorizeBand5;

//block2 filters
uniform	float block2BlurAmount;
uniform	float block2BlurRadius;
uniform	float block2SharpenAmount;
uniform	float block2SharpenRadius;
uniform float block2FiltersBoost;
uniform float block2Dither;
uniform int block2DitherSwitch;
uniform int block2DitherType;


//final mix
uniform float finalMixAmount;
uniform vec3 finalKeyValue;
uniform float finalKeyThreshold;
uniform float finalKeySoft;
uniform int finalMixType;
uniform int finalMixOverflow;
uniform int finalKeyOrder;

//matrix mixer
uniform int matrixMixType;
uniform int matrixMixOverflow;
uniform vec3 bgRGBIntoFgRed;
uniform vec3 bgRGBIntoFgGreen;
uniform vec3 bgRGBIntoFgBlue;





in vec2 texCoordVarying;

out vec4 outputColor;

//color space conversions
vec3 rgb2hsb(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsb2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

//general signal utilities
float wrap(float inColor){

	if(inColor<0.0){
		inColor=1.0-abs(inColor);
	}

	if(inColor>1.0){
		inColor=fract(inColor);
	}

	return inColor;
}


float foldover(float inColor){

	if(inColor<0.0){
		inColor=abs(inColor);
	}

	if(inColor>1.0){
		inColor=1.0-(fract(inColor));
	}

	if(inColor<0.0){
		inColor=abs(inColor);
	}
	return inColor;
}

float mirror(float a){
	if(a > 0.0) return a;
	return -(1.0 + a);
}

vec2 wrapCoord(vec2 coord){

    //if(abs(coord.x)>width){coord.x=abs(width-coord.x);}
    //if(abs(coord.y)>height){coord.y=abs(height-coord.y);}

    coord.x=mod(coord.x,width);
    coord.y=mod(coord.y,height);

    return coord;
}

vec2 mirrorCoord(vec2 coord){

    float widthLess=width-1.0;
	float heightLess=height-1.0;

    coord.x=(widthLess)-mirror(mod(coord.x,2.0*widthLess)-widthLess);
    coord.y=(heightLess)-mirror(mod(coord.y,2.0*heightLess)-heightLess);

    return coord;
}


vec2 rotate(vec2 coord,float theta,int mode){

	vec2 rotate_coord=vec2(0,0);
	//this version does not preserve aspect ratio
	if(mode==0){
		vec2 center_coord=vec2(coord.x-width/2,coord.y-height/2);
		float spiral=abs(coord.x+coord.y)/2*width;
		coord.x=spiral+coord.x;
		coord.y=spiral+coord.y;
		rotate_coord.x=center_coord.x*cos(theta)-center_coord.y*sin(theta);
		rotate_coord.y=center_coord.x*sin(theta)+center_coord.y*cos(theta);

		rotate_coord=rotate_coord+vec2(width/2,height/2);
	}
	//so lets try one that does and see what happens
	//so this works, can try switchable versions for a bit and see how one feels
	//while this preserves the aspect ratio of what feeds into it,
	//the other version is always circular which is def fun a lot of time.
	if(mode==1){
		vec2 center_coord=vec2((coord.x-width/2)*inverseWidth,(coord.y-height/2)*inverseHeight);
		float spiral=abs(coord.x+coord.y)/2*width;
		coord.x=spiral+coord.x;
		coord.y=spiral+coord.y;
		rotate_coord.x=center_coord.x*cos(theta)-center_coord.y*sin(theta);
		rotate_coord.y=center_coord.x*sin(theta)+center_coord.y*cos(theta);

		//rotate_coord=rotate_coord+vec2(width/2,height/2);
		rotate_coord.x=width*rotate_coord.x+width/2;
		rotate_coord.y=height*rotate_coord.y+height/2;
	}

    return rotate_coord;


}//endrotate

vec2 kaleidoscope(vec2 inCoord, float segment, float slice){
	if(segment>0.0){
	//question: can we rotate the coords in here to select which portion of the screen we
	//are kaleidoscoping from??
		inCoord=rotate(inCoord,slice,1);

		inCoord.x=inCoord.x/width;
		inCoord.y=inCoord.y/height;

		inCoord=2.0*inCoord-1.0;

		float radius=sqrt( dot(inCoord,inCoord) );
		float angle=atan(inCoord.y,inCoord.x);
		float segmentAngle=TWO_PI/segment;
		angle-=segmentAngle*floor(angle/segmentAngle);
		angle=min(angle,segmentAngle-angle);
		inCoord=radius*vec2(cos(angle),sin(angle));

		inCoord=.5*(inCoord+1.0);
		inCoord.x*=width;
		inCoord.y*=height;

		inCoord=rotate(inCoord,-slice,0);
	}
	return inCoord;
}

//takes in coords and shears them
vec2 shear(vec2 inCoord, vec4 shearMatrix){
	inCoord.x-=width/2.0;
	inCoord.y-=height/2.0;

	//vec2 outCoord=vec2(0.0,0.0);

	inCoord.x=shearMatrix.x*inCoord.x+shearMatrix.y*inCoord.y;
	inCoord.y=shearMatrix.z*inCoord.x+shearMatrix.w*inCoord.y;

	inCoord.x+=width/2.0;
	inCoord.y+=height/2.0;
	return inCoord;
}


//MIX
//fg is foreground color, bg is background
//mixmodes 0=lerp, 1=add/subtrac, 2=mult, 3=dodge
//mixOverflowModes 0-clamp,1 wrap,2 fold?;
vec4 mixnKeyVideo(vec4 fg, vec4 bg, float amount, int selectMixMode, float keyThreshold,
float keySoft,vec3 keyValue,int keyOrder,int mixOverflow,vec4 mask,int keyMaskSwitch){

	vec4 outColor=vec4(0.0,0.0,0.0,1.0);
	//lerp
	if(selectMixMode==0){
		outColor=mix(fg, bg, amount);
	}
	//addsubtrack
	if(selectMixMode==1){
		outColor.rgb=fg.rgb+amount*bg.rgb;
	}

	//difference
	if(selectMixMode==2){
		outColor.rgb=abs(fg.rgb-amount*bg.rgb);
	}

	//mult
	if(selectMixMode==3){
		outColor.rgb=mix(fg.rgb,fg.rgb*bg.rgb,amount);
	}

	//dodge
	if(selectMixMode==4){
		outColor.rgb=mix(fg.rgb,fg.rgb/(1.00001-bg.rgb),amount);
	}

	//clamp, wrap, fold
	if(mixOverflow==0){
		outColor.rgb=clamp(outColor.rgb,0.0,1.0);
	}

	if(mixOverflow==1){
		//outColor.rgb=fract(abs(outColor.rgb));
		outColor.r=wrap(outColor.r);
		outColor.g=wrap(outColor.g);
		outColor.b=wrap(outColor.b);
	}

	if(mixOverflow==2){
	//foldover
		outColor.r=foldover(outColor.r);
		outColor.g=foldover(outColor.g);
		outColor.b=foldover(outColor.b);
	}

	float chromaDistance=distance(keyValue,fg.rgb);
	float lower=chromaDistance-keyThreshold;

	if( chromaDistance < keyThreshold ){
		//outColor=mix(bg,fg,keySoft*(abs(keyValue-lower)/lower));

		//i don't quite think this is working, lets try something else some
		//other time...I think the way to do it is to have generated a blur earlier in the
		//chain, and use the blurred value to
		outColor=mix(bg,outColor,keySoft*abs(1.0-(chromaDistance-keyThreshold)));
		//outColor=bg;
		//outColor=mix(bg,fg,keySoft*abs(1.0-(chromaDistance-keyThreshold)));
	 }


	//taking for granted that the mask is in greyscale so any rgb value can be
	//used as test
	//starting off with 1 (white) returns fg and 0 (black) returns bg
	if(keyMaskSwitch==1){
		outColor=mix(outColor,bg,mask.g);
	}
	return outColor;
}


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


void main()
{

	//BLOCK1
	vec2 block1Coords=texCoordVarying*vec2(width,height);

	//GEO
	vec2 center=vec2(width/2.0,height/2.0);
	//geometry
	if(block1HFlip==1){
		block1Coords.x=width-block1Coords.x;
	}
	if(block1VFlip==1){
		block1Coords.y=height-block1Coords.y;
	}

	if(block1HMirror==1){
        if(block1Coords.x>width/2){block1Coords.x=abs(width-block1Coords.x);}
    }//endifhflip1
    if(block1VMirror==1){
        if(block1Coords.y>height/2){block1Coords.y=abs(height-block1Coords.y);}
    }//endifvflip1

	block1Coords=kaleidoscope(block1Coords,block1KaleidoscopeAmount,block1KaleidoscopeSlice);

	block1Coords+=block1XYDisplace;
	block1Coords-=center;
	block1Coords*=block1ZDisplace;
	block1Coords+=center;

	block1Coords=rotate(block1Coords,block1Rotate,block1RotateMode);

	block1Coords=shear(block1Coords,block1ShearMatrix);

	if(block1GeoOverflow==1){block1Coords=wrapCoord(block1Coords);}
	if(block1GeoOverflow==2){block1Coords=mirrorCoord(block1Coords);}



	vec4 block1Color=blurAndSharpen(block1Output,(block1Coords/vec2(width,height)),block1SharpenAmount,block1SharpenRadius,
		block1FiltersBoost,block1BlurRadius,block1BlurAmount);

	if(block1GeoOverflow==0){
		if(block1Coords.x>width || block1Coords.y> height || block1Coords.x<0.0 || block1Coords.y<0.0){
			block1Color=vec4(0.0);
		}
	}

	//vec4 block1Color=texture(block1Output, block1Coords/vec2(width,height));

	vec3 block1ColorHSB=rgb2hsb(block1Color.rgb);

	//BLOCK1 COLORIZE - Optimized to reduce branching
	// Pre-calculate both HSB and RGB modes, then select with mix()
	vec3 colorizedHSB1 = block1ColorizeBand1 + vec3(0.0, 0.0, block1ColorHSB.z);
	vec3 colorizedHSB2 = block1ColorizeBand2 + vec3(0.0, 0.0, block1ColorHSB.z);
	vec3 colorizedHSB3 = block1ColorizeBand3 + vec3(0.0, 0.0, block1ColorHSB.z);
	vec3 colorizedHSB4 = block1ColorizeBand4 + vec3(0.0, 0.0, block1ColorHSB.z);
	vec3 colorizedHSB5 = block1ColorizeBand5 + vec3(0.0, 0.0, block1ColorHSB.z);
	
	vec3 hsbMode1 = hsb2rgb(colorizedHSB1);
	vec3 hsbMode2 = hsb2rgb(colorizedHSB2);
	vec3 hsbMode3 = hsb2rgb(colorizedHSB3);
	vec3 hsbMode4 = hsb2rgb(colorizedHSB4);
	vec3 hsbMode5 = hsb2rgb(colorizedHSB5);
	
	vec3 rgbMode1 = block1ColorizeBand1 + block1Color.rgb;
	vec3 rgbMode2 = block1ColorizeBand2 + block1Color.rgb;
	vec3 rgbMode3 = block1ColorizeBand3 + block1Color.rgb;
	vec3 rgbMode4 = block1ColorizeBand4 + block1Color.rgb;
	vec3 rgbMode5 = block1ColorizeBand5 + block1Color.rgb;
	
	// Select mode based on switch (0=HSB, 1=RGB) using mix()
	float modeMix = float(block1ColorizeHSB_RGB);
	vec3 colorizedRGB1 = mix(hsbMode1, rgbMode1, modeMix);
	vec3 colorizedRGB2 = mix(hsbMode2, rgbMode2, modeMix);
	vec3 colorizedRGB3 = mix(hsbMode3, rgbMode3, modeMix);
	vec3 colorizedRGB4 = mix(hsbMode4, rgbMode4, modeMix);
	vec3 colorizedRGB5 = mix(hsbMode5, rgbMode5, modeMix);
	
	// Band selection using smoothstep for interpolation between bands
	float brightness = block1ColorHSB.z;
	float bandMix1 = clamp(brightness * 4.0, 0.0, 1.0);                    // 0.0-0.25 range
	float bandMix2 = clamp((brightness - 0.25) * 4.0, 0.0, 1.0);           // 0.25-0.5 range  
	float bandMix3 = clamp((brightness - 0.5) * 4.0, 0.0, 1.0);            // 0.5-0.75 range
	float bandMix4 = clamp((brightness - 0.75) * 4.0, 0.0, 1.0);           // 0.75-1.0 range
	
	// Select which band range we're in using step functions
	float inBand1 = 1.0 - step(0.25, brightness);
	float inBand2 = step(0.25, brightness) * (1.0 - step(0.5, brightness));
	float inBand3 = step(0.5, brightness) * (1.0 - step(0.75, brightness));
	float inBand4 = step(0.75, brightness);
	
	// Combine bands with proper mixing - each region only contributes in its range
	vec3 colorizedRGB = mix(
		mix(
			mix(colorizedRGB1, colorizedRGB2, bandMix1),
			mix(colorizedRGB2, colorizedRGB3, bandMix2),
			step(0.25, brightness)
		),
		mix(colorizedRGB3, colorizedRGB4, bandMix3),
		step(0.5, brightness)
	);
	colorizedRGB = mix(colorizedRGB, mix(colorizedRGB4, colorizedRGB5, bandMix4), step(0.75, brightness));
	
	// Apply colorization only when switch is enabled
	float enableMix = float(block1ColorizeSwitch);
	block1Color.rgb = mix(block1Color.rgb, colorizedRGB, enableMix);

	//dither - branchless version
	float ditherEnable = float(block1DitherSwitch);
	block1Color.r = mix(block1Color.r, dither2(block1Color.r, block1Coords, block1Dither, block1DitherType), ditherEnable);
	block1Color.g = mix(block1Color.g, dither2(block1Color.g, block1Coords, block1Dither, block1DitherType), ditherEnable);
	block1Color.b = mix(block1Color.b, dither2(block1Color.b, block1Coords, block1Dither, block1DitherType), ditherEnable);
	/*
	vec2 block2Coords=texCoordVarying*vec2(width,height);
	vec4 block2Color=texture(block2Output, block2Coords/vec2(width,height));
	*/



	//block2
	vec2 block2Coords=texCoordVarying*vec2(width,height);

	//GEO
	//vec2 center=vec2(640,360);
	//geometry
	if(block2HFlip==1){
		block2Coords.x=width-block2Coords.x;
	}
	if(block2VFlip==1){
		block2Coords.y=height-block2Coords.y;
	}

	if(block2HMirror==1){
        if(block2Coords.x>width/2){block2Coords.x=abs(width-block2Coords.x);}
    }//endifhflip1
    if(block2VMirror==1){
        if(block2Coords.y>height/2){block2Coords.y=abs(height-block2Coords.y);}
    }//endifvflip1

	block2Coords=kaleidoscope(block2Coords,block2KaleidoscopeAmount,block2KaleidoscopeSlice);

	block2Coords+=block2XYDisplace;
	block2Coords-=center;
	block2Coords*=block2ZDisplace;
	block2Coords+=center;

	block2Coords=rotate(block2Coords,block2Rotate,block2RotateMode);

	block2Coords=shear(block2Coords,block2ShearMatrix);

	if(block2GeoOverflow==1){block2Coords=wrapCoord(block2Coords);}
	if(block2GeoOverflow==2){block2Coords=mirrorCoord(block2Coords);}



	vec4 block2Color=blurAndSharpen(block2Output,(block2Coords/vec2(width,height)),block2SharpenAmount,block2SharpenRadius,
		block2FiltersBoost,block2BlurRadius,block2BlurAmount);

	if(block2GeoOverflow==0){
		if(block2Coords.x>width || block2Coords.y> height || block2Coords.x<0.0 || block2Coords.y<0.0){
			block2Color=vec4(0.0);
		}
	}






	vec3 block2ColorHSB=rgb2hsb(block2Color.rgb);

	//block2 COLORIZE - Optimized to reduce branching
	// Pre-calculate both HSB and RGB modes, then select with mix()
	vec3 b2_colorizedHSB1 = block2ColorizeBand1 + vec3(0.0, 0.0, block2ColorHSB.z);
	vec3 b2_colorizedHSB2 = block2ColorizeBand2 + vec3(0.0, 0.0, block2ColorHSB.z);
	vec3 b2_colorizedHSB3 = block2ColorizeBand3 + vec3(0.0, 0.0, block2ColorHSB.z);
	vec3 b2_colorizedHSB4 = block2ColorizeBand4 + vec3(0.0, 0.0, block2ColorHSB.z);
	vec3 b2_colorizedHSB5 = block2ColorizeBand5 + vec3(0.0, 0.0, block2ColorHSB.z);
	
	vec3 b2_hsbMode1 = hsb2rgb(b2_colorizedHSB1);
	vec3 b2_hsbMode2 = hsb2rgb(b2_colorizedHSB2);
	vec3 b2_hsbMode3 = hsb2rgb(b2_colorizedHSB3);
	vec3 b2_hsbMode4 = hsb2rgb(b2_colorizedHSB4);
	vec3 b2_hsbMode5 = hsb2rgb(b2_colorizedHSB5);
	
	vec3 b2_rgbMode1 = block2ColorizeBand1 + block2Color.rgb;
	vec3 b2_rgbMode2 = block2ColorizeBand2 + block2Color.rgb;
	vec3 b2_rgbMode3 = block2ColorizeBand3 + block2Color.rgb;
	vec3 b2_rgbMode4 = block2ColorizeBand4 + block2Color.rgb;
	vec3 b2_rgbMode5 = block2ColorizeBand5 + block2Color.rgb;
	
	// Select mode based on switch (0=HSB, 1=RGB) using mix()
	float b2_modeMix = float(block2ColorizeHSB_RGB);
	vec3 b2_colorizedRGB1 = mix(b2_hsbMode1, b2_rgbMode1, b2_modeMix);
	vec3 b2_colorizedRGB2 = mix(b2_hsbMode2, b2_rgbMode2, b2_modeMix);
	vec3 b2_colorizedRGB3 = mix(b2_hsbMode3, b2_rgbMode3, b2_modeMix);
	vec3 b2_colorizedRGB4 = mix(b2_hsbMode4, b2_rgbMode4, b2_modeMix);
	vec3 b2_colorizedRGB5 = mix(b2_hsbMode5, b2_rgbMode5, b2_modeMix);
	
	// Band selection using smoothstep for interpolation between bands
	float b2_brightness = block2ColorHSB.z;
	float b2_bandMix1 = clamp(b2_brightness * 4.0, 0.0, 1.0);
	float b2_bandMix2 = clamp((b2_brightness - 0.25) * 4.0, 0.0, 1.0);
	float b2_bandMix3 = clamp((b2_brightness - 0.5) * 4.0, 0.0, 1.0);
	float b2_bandMix4 = clamp((b2_brightness - 0.75) * 4.0, 0.0, 1.0);
	
	// Combine bands with proper mixing
	vec3 b2_colorizedRGB = mix(
		mix(
			mix(b2_colorizedRGB1, b2_colorizedRGB2, b2_bandMix1),
			mix(b2_colorizedRGB2, b2_colorizedRGB3, b2_bandMix2),
			step(0.25, b2_brightness)
		),
		mix(b2_colorizedRGB3, b2_colorizedRGB4, b2_bandMix3),
		step(0.5, b2_brightness)
	);
	b2_colorizedRGB = mix(b2_colorizedRGB, mix(b2_colorizedRGB4, b2_colorizedRGB5, b2_bandMix4), step(0.75, b2_brightness));
	
	// Apply colorization only when switch is enabled
	float b2_enableMix = float(block2ColorizeSwitch);
	block2Color.rgb = mix(block2Color.rgb, b2_colorizedRGB, b2_enableMix);

	//dither - branchless version
	float b2_ditherEnable = float(block2DitherSwitch);
	block2Color.r = mix(block2Color.r, dither2(block2Color.r, block2Coords, block2Dither, block2DitherType), b2_ditherEnable);
	block2Color.g = mix(block2Color.g, dither2(block2Color.g, block2Coords, block2Dither, block2DitherType), b2_ditherEnable);
	block2Color.b = mix(block2Color.b, dither2(block2Color.b, block2Coords, block2Dither, block2DitherType), b2_ditherEnable);



	//matrix mixer test
	//default atm: bg=BLOCK2, fg=BLOCK1
	vec3 matrixMixOut=vec3(0.0,0.0,0.0);
	vec3 fg=block1Color.rgb;
	vec3 bg=block2Color.rgb;

	if(finalKeyOrder==1){

		fg=block2Color.rgb;
		bg=block1Color.rgb;
	}
	vec3 fgR=vec3(fg.r,fg.r,fg.r);
	vec3 fgG=vec3(fg.g,fg.g,fg.g);
	vec3 fgB=vec3(fg.b,fg.b,fg.b);

	vec3 scaleVec=vec3(.33,.33,.33);

	//lerp
	if(matrixMixType==0){
		matrixMixOut.r=dot( mix(fgR,bg,bgRGBIntoFgRed ) , scaleVec );
		matrixMixOut.g=dot( mix(fgG,bg,bgRGBIntoFgGreen) , scaleVec );
		matrixMixOut.b=dot( mix(fgB,bg,bgRGBIntoFgBlue) , scaleVec );
	}
	//add
	if(matrixMixType==1){
		matrixMixOut.r=dot( fgR+bgRGBIntoFgRed*bg , scaleVec );
		matrixMixOut.g=dot( fgG+bgRGBIntoFgGreen*bg , scaleVec );
		matrixMixOut.b=dot( fgB+bgRGBIntoFgBlue*bg , scaleVec );
	}
	//diff
	if(matrixMixType==2){
		matrixMixOut.r=dot( abs(fgR-bgRGBIntoFgRed*bg) , scaleVec );
		matrixMixOut.g=dot( abs(fgG-bgRGBIntoFgGreen*bg) , scaleVec );
		matrixMixOut.b=dot( abs(fgB-bgRGBIntoFgBlue*bg) , scaleVec );
	}
	//mult
	if(matrixMixType==3){
		matrixMixOut.r=dot( mix(fgR,bg*fgR,bgRGBIntoFgRed ) , scaleVec );
		matrixMixOut.g=dot( mix(fgG,bg*fgG,bgRGBIntoFgGreen) , scaleVec );
		matrixMixOut.b=dot( mix(fgB,bg*fgB,bgRGBIntoFgBlue) , scaleVec );
	}
	//dodge
	if(matrixMixType==4){
		matrixMixOut.r=dot( mix(fgR,bg/(1.00001 - fgR),bgRGBIntoFgRed ) , scaleVec );
		matrixMixOut.g=dot( mix(fgG,bg/(1.00001 - fgG),bgRGBIntoFgGreen) , scaleVec );
		matrixMixOut.b=dot( mix(fgB,bg/(1.00001 - fgB),bgRGBIntoFgBlue) , scaleVec );
	}


	if(matrixMixOverflow==0){
		matrixMixOut=clamp(matrixMixOut,0.0,1.0);
	}
	if(matrixMixOverflow==1){
		matrixMixOut.r=wrap(matrixMixOut.r);
		matrixMixOut.g=wrap(matrixMixOut.g);
		matrixMixOut.b=wrap(matrixMixOut.b);
	}
	if(matrixMixOverflow==2){
		matrixMixOut.r=foldover(matrixMixOut.r);
		matrixMixOut.g=foldover(matrixMixOut.g);
		matrixMixOut.b=foldover(matrixMixOut.b);
	}

	vec4 outColor=mixnKeyVideo(vec4(matrixMixOut,1.0),vec4(bg,1.0),finalMixAmount,finalMixType,finalKeyThreshold,finalKeySoft
					,finalKeyValue,finalKeyOrder,finalMixOverflow,vec4(0.0,0.0,0.0,0.0),0);

	outputColor=outColor;
}
