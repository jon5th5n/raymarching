#version 410
precision lowp float;
vec4 sdf(vec3 point);

varying vec2 uv;

uniform ivec2 cam_size;
uniform float cam_fov;
uniform float cam_depth;
uniform float cam_threshold;
uniform vec3 cam_position;
uniform vec3 cam_direction;
uniform vec3 cam_up;
uniform vec3 cam_right;

vec3 rotate(vec3 v,vec3 rotation_axis,float angle){
    rotation_axis=normalize(rotation_axis);
    
    mat3 k_mat=mat3(0.,-rotation_axis.z,rotation_axis.y,
        rotation_axis.z,0.,-rotation_axis.x,
    -rotation_axis.y,rotation_axis.x,0.);
    
    mat3 i_mat=mat3(1.,0.,0.,
        0.,1.,0.,
    0.,0.,1.);
    
    mat3 rot_mat=i_mat+sin(angle)*k_mat+(1.-cos(angle))*(k_mat*k_mat);
    
    return rot_mat*v;
}

vec3 get_normal3(vec3 p){
    float epsilon=cam_threshold;
    float centerDistance=sdf(p).w;
    float xDistance=sdf(p+vec3(epsilon,0,0)).w;
    float yDistance=sdf(p+vec3(0,epsilon,0)).w;
    float zDistance=sdf(p+vec3(0,0,epsilon)).w;
    vec3 normal=(vec3(xDistance,yDistance,zDistance)-centerDistance)/epsilon;
    
    return normal;
}

vec3 get_normal6(in vec3 p)
{
    const vec3 small_step=vec3(.001,0.,0.);
    
    float gradient_x=sdf(p+small_step.xyy).w-sdf(p-small_step.xyy).w;
    float gradient_y=sdf(p+small_step.yxy).w-sdf(p-small_step.yxy).w;
    float gradient_z=sdf(p+small_step.yyx).w-sdf(p-small_step.yyx).w;
    
    vec3 normal=vec3(gradient_x,gradient_y,gradient_z);
    
    return normalize(normal);
}

#define NUM_LIGHTS 1
vec3 lights[NUM_LIGHTS]=vec3[](vec3(-50.,0.,200.));

float get_shading(vec3 point){
    // float tmp=.5;
    // float shading=1.;
    // for(uint i=0;i<NUM_LIGHTS;i++){
        //     vec3 light=lights[i];
        //     float light_dist=length(light-point);
        //     vec3 ray=normalize(light-point);
        
        //     //march
        //     float first_dist=sdf(point).w;
        //     float travel_dist=0.;
        //     while(travel_dist<light_dist){
            //         float dist=sdf(point+(ray*travel_dist)).w;
            
            //         if(dist<=cam_threshold){
                //             shading=0.;
                //             break;
            //         }
            
            //         travel_dist+=dist;
            
            //         shading=min(shading,dist/(travel_dist*.02));
        //     }
        
    // }
    
    // return shading;
    float shadow=1.f;
    float shadowRayLength=0.;
    
    vec3 light=lights[0];
    float light_dist=length(light-point);
    vec3 ray=normalize(light-point);
    
    while(shadowRayLength<light_dist)
    {
        vec3 testPos=point+ray*shadowRayLength;
        float distFromObject=sdf(testPos).w;
        shadowRayLength+=distFromObject;
        
        shadow=min(shadow,distFromObject/(shadowRayLength*.01));
        
        if(distFromObject<cam_threshold)
        {
            shadow=0.;
            break;
        }
    }
    
    return shadow/(sqrt(light_dist/100));
}

vec4 march(){
    vec2 nuv=(uv*2.)-1.;
    
    float fovh=cam_fov;
    float fovv=cam_fov*(float(cam_size.y)/float(cam_size.x));
    
    vec3 ray=rotate(rotate(cam_direction,cam_up,(fovh/2.)*-nuv.x),cam_right,(fovv/2.)*-nuv.y);
    
    //-----
    
    vec4 color=vec4(.53,.81,.92,1.);
    float i=0.;
    while(i<=cam_depth){
        vec3 point=cam_position+(ray*i);
        vec4 color_dist=sdf(point);
        float dist=color_dist.w;
        if(dist<=cam_threshold){
            float shading=get_shading(point-(ray*(cam_threshold+dist+cam_threshold*.2)));
            vec4 c=vec4(color_dist.xyz*shading,1.);
            float d_fog=i/cam_depth;
            color=mix(c,color,d_fog);
            break;
        }
        
        i+=dist;
    }
    
    return color;
}

void main(){
    gl_FragColor=march();
}