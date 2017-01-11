
typedef struct Camera
{
    float3 pos;
    float3 target;
    float focal_distance;
    float3 e;
    float3 p1;
    float3 p2;
    float3 p3;
    float3 up;
    float3 right;
    int width;
    int height;
    float aspect_ratio;
    float lens_size;

} Camera;


typedef struct Ray
{
    float3 o;
    float3 d;
    float t;
    float3 n;
    size_t obj_idx;
    bool inside;
} Ray;

typedef struct Material
{
    float refl;
    float refr;
    bool emissive;
    float3 diffuse;
    bool checkerboard;
} Material;

typedef struct Sphere
{
    float3 pos;
    float r;
} Sphere;

#define PI (3.1415926535359f)
#define INVPI (1.0f / PI)



// TODO, copy the scene stuff from scene.cs

#define LIGHTSCALE (1.0f)
#define SCENE_SIZE (9)



#define LIGHTSIZE (0.3f)


typedef struct Scene
{
	Sphere spheres[SCENE_SIZE];
	Material materials[SCENE_SIZE];	
} Scene;

const inline Scene create_scene(void)
{
	Scene scene;

	scene.spheres[0].pos.x = 0.0f;
	scene.spheres[0].pos.y = -4999.0f;
	scene.spheres[0].pos.z = 0.0f;
	scene.spheres[0].r     = 4998.5f * 4998.5f;
	scene.materials[0].checkerboard = true;

	scene.spheres[1].pos.x = 0.0f;
	scene.spheres[1].pos.y = 0.0f;
	scene.spheres[1].pos.z = -5000.0f;
	scene.spheres[1].r     = 4993.0f * 4993.0f;
	scene.materials[1].diffuse = (float3)(1.0f,1.0f,1.0f);

	scene.spheres[2].pos = (float3)(-0.8f,0.0f,-2.0f);
	scene.spheres[2].r   = 0.3f * 0.3f;
	scene.materials[2].refl = 0.8f;
	scene.materials[2].diffuse = (float3)(1.0f, 0.2f, 0.2f);

	scene.spheres[3].pos = (float3)(0.0f,0.0f,-2.0f);
	scene.spheres[3].r   = 0.3f * 0.3f;
	scene.materials[3].refr = 1.0f;
	scene.materials[3].diffuse = (float3)(0.9f,1.0f,0.9f);

	scene.spheres[4].pos = (float3)(0.8f, 0.0f, -2.0f);
	scene.spheres[4].r   = 0.3f * 0.3f;
	scene.materials[4].refl = 0.8f;
	scene.materials[4].diffuse = (float3)(0.2f,0.2f,1.0f);

	scene.spheres[5].pos = (float3)(-0.8f,-0.8f,-2.0f);
	scene.spheres[5].r = 0.5f * 0.5f;
	scene.materials[5].diffuse = (float3)(1.0f,1.0f,1.0f);

	scene.spheres[6].pos = (float3)(0.0f,-0.8f,-2.0f);
	scene.spheres[6].r   = 0.5f*0.5f;
	scene.materials[6].diffuse = (float3)(1.0f,1.0f,1.0f);

	scene.spheres[7].pos = (float3)(0.8f,-0.8f,-2.0f);
	scene.spheres[7].r = 0.5f*0.5f;
	scene.materials[7].diffuse = (float3)(1.0f,1.0f,1.0f);

	scene.spheres[8].pos = (float3)(2.7f,1.7f,-0.5f);
	scene.spheres[8].r = LIGHTSIZE * LIGHTSIZE;
	scene.materials[8].emissive = true;
	scene.materials[8].diffuse = (float3)(8.5f * LIGHTSCALE, 8.5f * LIGHTSCALE, 8.5f * LIGHTSCALE);

	return scene;
}

float3 sampleSkydome(float3 d, global float *skybox) {
	int u = (int)(2500.0f * 0.5f * (1.0f + atan2(d.x, -d.z) * INVPI));
	int v = (int)(1250.0f * acos(d.y) * INVPI);
	int idx = u + v * 2500;
	return (float3)(skybox[idx * 3 + 0], skybox[idx * 3 + 1], skybox[idx * 3 + 2]);
}


Material get_material(Material mat, float3 i)
{
    if(mat.checkerboard)
    {
        int tx = ((int)(i.x * 3.0f + 1000) + (int)(i.z*3.0f+1000)) & 1;
        
        Material mat2;
        mat2.diffuse = tx == 1 ? (float3)(1.0f,1.0f,1.0f) : (float3)(0.2f,0.2f,0.2f);
        
        return mat2;
    }
    return mat;
}


uint wang_hash(uint seed)
{
	seed = (seed ^ 61) ^ (seed >> 16);
    seed *= 9;
    seed = seed ^ (seed >> 4);
    seed *= 0x27d4eb2d;
    seed = seed ^ (seed >> 15);
    return seed;
}
/* XOR 128 */
#define XOR128_m_FP (4294967296.0f)
#define XOR_min_FP (1.0f/XOR128_m_FP)
uint next(uint4 *ctx)
{
    uint t = ctx->x ^ (ctx->x << 11);
    *ctx = ctx->yzww;
    ctx->w = ctx->w ^ (ctx->w >> 19) ^ (t ^ (t >> 8));

    return ctx->w;
}

float next_float(uint4 *ctx)
{
    return (float)(next(ctx)) * XOR_min_FP;
}

float3 reflect(float3 V, float3 N){
	return V - 2.0f * dot( V, N ) * N;
}

float3 refraction(uint4 *rng, bool inside, float3 D, float3 N, float3 R )
{
	float nc;
	float nt;
	if(inside) { 
		nc = 1.0f;
		nt = 1.2f;
	} else {
		nc = 1.2f;
		nt = 1.0f;
	}
	float nnt = nt / nc;
	float ddn = dot( D, N ); 
	float cos2t = 1.0f - nnt * nnt * (1 - ddn * ddn);
	R = reflect( D, N );
	if (cos2t >= 0)
	{
		float r1 = next_float(rng);
		float a = nt - nc;
		float b = nt + nc;
		float R0 = a * a / (b * b);
		float c = 1 + ddn;
		float Tr = 1 - (R0 + (1 - R0) * c * c * c * c * c);
		if (r1 < Tr) {
			R = (D * nnt - N * (ddn * nnt + (float)sqrt(cos2t)));
		}
	}
	return R;
}

float3 diffuse_reflection(uint4 *rng, float3 N )
{
	float r1 = next_float(rng);
	float r2 = next_float(rng);
	float r = sqrt( 1.0f - r1 * r1 );
	float phi = 2 * PI * r2;
	float3 R = (float3)(
		cos( phi ) * r,
		sin( phi ) * r,
		r1);
	if (dot( N, R ) < 0) {
		R *= -1.0f;
	}
	return R;
}


Ray gen_ray(Camera *cam, uint4 *rng, size_t x, size_t y)
{
    float r0 = next_float(rng);
    float r1 = next_float(rng);
    float r2 = next_float(rng);
    float r3 = next_float(rng);
	int width = cam->width;
	int height = cam->height;
    float u = ((float) x + r0) / (float)(width);
    float v = ((float) y + r1) / (float)(height);
    float3 t = cam->p1 + u * (cam->p2 - cam->p1) + v * (cam->p3 - cam->p1);
    float3 p = cam->pos + cam->lens_size * (r2 * cam->right + r3 * cam->up);
    float3 d = normalize(t-p);
    
    Ray ray =
    {
        .o = p,
        .d = d,
        .t = 1e34f,
        .obj_idx = -1,
        .inside = false
    };
    return ray;
}

// intersects our ray with objects in our scene

void intersect_sphere(int i, Ray *ray,  const Scene *scene)
{
		Sphere sphere = scene->spheres[i];
		float3 L = sphere.pos - ray->o;
		float tca = dot(L, ray->d);
		if (tca < 0) {
			return;
		}
		float d2 = dot(L, L) - tca * tca;
		if (d2 > sphere.r) {
			return;
		}
		float thc = sqrt(sphere.r - d2);
		float t0 = tca - thc;
		float t1 = tca + thc;
		if (t0 > 0)
		{
			if (t0 > ray->t) {
				return;
			}
			ray->n = normalize(ray->o + t0 * ray->d - sphere.pos);
			ray->obj_idx = i;
			ray->t = t0;
		}
		else
		{
			if ((t1 > ray->t) || (t1 < 0)) {
				return;
			}
			ray->n = normalize(sphere.pos - (ray->o + t1 * ray->d));
			ray->obj_idx = i;
			ray->t = t1;
		}
}



void intersect(Ray *ray, const  Scene *scene)
{
	ray->obj_idx = -1;
	intersect_sphere(0, ray, scene);
	intersect_sphere(1, ray, scene);
	intersect_sphere(2, ray, scene);
	intersect_sphere(3, ray, scene);
	intersect_sphere(4, ray, scene);
	intersect_sphere(5, ray, scene);
	intersect_sphere(6, ray, scene);
	intersect_sphere(7, ray, scene);
	intersect_sphere(8, ray, scene);
}

#define MAXDEPTH (20)
#define EPSILON (0.0001f)
float3 sample(Ray *ray, uint4 *rng, const  Scene *scene, global float *skybox)
{
	float3 sample = (float3)(1.0f, 1.0f, 1.0f);;

	for (int depth = 0; depth < MAXDEPTH; depth++)
	{
               
		intersect(ray, scene);

		int idx = ray->obj_idx;

		//return (float3)((((float)idx+1)/9.0f), (((float)idx+1)/9.0f), (((float)idx+1)/9.0f));
		

		if (ray->obj_idx == -1)
		{
			sample *=  1.0f * sampleSkydome(ray->d, skybox);
			return sample;
		}

		float3 I = ray->o + ray->t * ray->d;


		Material material = get_material(scene->materials[ray->obj_idx],I);
		   
		if (material.emissive)
		{
			sample *= 1.0f * material.diffuse;
			return sample;
		}

		float r0 = next_float(rng);
		float3 R = (float3)(0.0f, 0.0f, 0.0f);
		
		if (r0 < material.refr)
		{
			R = refraction(rng, ray->inside, ray->d, ray->n, R);
			Ray extensionRay = { .o = I + R * EPSILON, .d = R,  .t = 1.0e34f};
			
			if (dot(ray->n, R) < 0) {
				extensionRay.inside = true;
			} else {
				extensionRay.inside = false;
			}

			*ray = extensionRay;
			sample *= material.diffuse;
		}
		else if ((r0 < (material.refl + material.refr)) && (depth < MAXDEPTH))
		{
			R = reflect(ray->d, ray->n);
			Ray n = { .o = I + R * EPSILON, .d = R,  .t = 1.0e34f};
			*ray = n;
			sample *= material.diffuse;
		}
		else
		{
			R = diffuse_reflection(rng, ray->n);
			Ray extensionRay = { .o = I + R * EPSILON, .d = R,  .t = 1.0e34f};
			sample *= dot(R, ray->n) * material.diffuse;
			*ray = extensionRay;
		}

	}
	return (float3)(0.0f, 0.0f, 0.0f);
}


uint float3_to_uint2(float x, float y, float z)
{
	int r = (int)(x*255.0f);
	int g = (int)(y*255.0f);
	int b = (int)(z*255.0f);
	return (r<<16)|(g<<8)|b;
}
#define BRIGHTNESS (1.5f)
uint float3_to_uint( float x, float y, float z )
{
	// apply gamma correction and convert to integer rgb
	uint r = (int)min( 255.0f, 256.0f * BRIGHTNESS * (float)sqrt( x ) );
	uint g = (int)min( 255.0f, 256.0f * BRIGHTNESS * (float)sqrt( y ) );
	uint b = (int)min( 255.0f, 256.0f * BRIGHTNESS * (float)sqrt( z ) );
	return (r << 16) + (g << 8) + b;
}


kernel void render(
    global float    *accum,
    int              spp,
    float            pos_x,
    float            pos_y,
    float            pos_z,
    float            target_x,
    float            target_y,
    float            target_z,
    float            focal_distance,
    float            e_x,
    float            e_y,
    float            e_z,
    float            p1_x,
    float            p1_y,
    float            p1_z,
    float            p2_x,
    float            p2_y,
    float            p2_z,
    float            p3_x,
    float            p3_y,
    float            p3_z,
    float            up_x,
    float            up_y,
    float            up_z,
    float            right_x,
    float            right_y,
    float            right_z,
    int              width,
    int              height,
    float            aspect_ratio,
    float            lens_size,
	global float	 *skybox,
	global uint		 *screen
)
{

    Camera camera =
    {
        .pos            = (float3)(pos_x,pos_y,pos_z),
        .target         = (float3)(target_x,target_y,target_z),
        .focal_distance = focal_distance,
        .e              = (float3)(e_x,e_y,e_z),
        .p1             = (float3)(p1_x,p1_y,p1_z),
        .p2             = (float3)(p2_x,p2_y,p2_z),
        .p3             = (float3)(p3_x,p3_y,p3_z),
        .up             = (float3)(up_x,up_y,up_z),
        .right          = (float3)(right_x,right_y,right_z),
        .width          = width,
        .height         = height,
        .aspect_ratio   = aspect_ratio,
        .lens_size      = lens_size
    };
    
    size_t y = get_global_id(0);
    size_t x = get_global_id(1);

	Scene scene = create_scene();


	uint j = x+y*get_global_size(1);
	uint seed = wang_hash(j);
	uint seed2 = wang_hash(spp);
    // seed the rng with thread ids and spp. Unique per frame

    uint4 rng;
    rng.x  = seed+seed2;
    rng.y  = seed+seed2+1;
    rng.z  = seed+seed2+2;
	rng.w  = seed+seed2+3;


    Ray ray = gen_ray(&camera, &rng, x, y);

	float3 result = sample(&ray,&rng,&scene, skybox);

	

	size_t i = (j)*3;
    accum[i] += result.x;
    accum[i+1] += result.y;
    accum[i+2] += result.z;
	screen[i/3] = float3_to_uint((1.0f/(float)spp)*accum[i], (1.0f/(float)spp)*accum[i+1], (1.0f/(float)spp)*accum[i+2]);

}

