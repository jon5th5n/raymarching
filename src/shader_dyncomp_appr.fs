precision lowp float;
vec4 sdf(vec3 point);

varying vec2 uv;

uniform ivec2 cam_size;
uniform float cam_fov;
uniform float cam_depth;
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
    float epsilon=.001;// arbitrary — should be smaller than any surface detail in your distance function, but not so small as to get lost in float precision
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

float get_shading(vec3 p){
    vec3 light_source=vec3(-15.,0.,50.);
    
    vec3 light_dir=light_source-p;
    
    return dot(get_normal3(p),light_dir);
}

vec4 march(){
    vec2 nuv=(uv*2.)-1.;
    
    float fovh=cam_fov;
    float fovv=cam_fov*(float(cam_size.y)/float(cam_size.x));
    
    vec3 ray=rotate(rotate(cam_direction,cam_up,(fovh/2.)*-nuv.x),cam_right,(fovv/2.)*-nuv.y);
    
    //-----
    
    vec4 color=vec4(.1804,.1804,.1804,1.);
    float i=0.;
    while(true){
        vec3 point=cam_position+(ray*i);
        vec4 color_dist=sdf(point);
        float dist=color_dist.w;
        if(dist<=.1){
            color=vec4(color_dist.xyz*get_shading(point)/70.,1.);
            break;
        }
        
        i+=dist;
        if(i>=cam_depth)break;
    }
    
    return color;
}

void main(){
    gl_FragColor=march();
}