precision lowp float;

//---------------------------------------------------------------------------------------

vec4 com_union(vec4 one,vec4 two){
    return one.w<two.w?one:two;
}
vec4 com_intersect(vec4 one,vec4 two){
    return one.w>two.w?one:two;
}
vec4 com_substract(vec4 one,vec4 two){
    return-one.w>two.w?vec4(one.xyz,-one.w):two;
}

//---------------------------------------------------------------------------------------

mat3 euler_to_rotation(float x,float y,float z){
    mat3 rx=mat3(1.,0.,0.,
        0.,cos(x),-sin(x),
        0.,sin(x),cos(x));
        
        mat3 ry=mat3(cos(y),0.,sin(y),
        0.,1.,0.,
        -sin(y),0.,cos(y));
        
        mat3 rz=mat3(cos(z),-sin(z),0.,
        sin(z),cos(z),0.,
    0.,0.,1.);
    
    return rz*ry*rx;
}

mat3 euler_to_rotation_ang(float x,float y,float z){
    return euler_to_rotation(x*.017453292,y*.017453292,z*.017453292);
}

vec4 sdf_sphere(vec3 point,vec3 color,float r){
    return vec4(color,length(point)-r);
}

vec4 sdf_box(vec3 point,vec3 color,vec3 s)
{
    vec3 q=abs(point)-s;
    return vec4(color,length(max(q,0.))+min(max(q.x,max(q.y,q.z)),0.));
}

vec4 sdf_torus(vec3 point,vec3 color,vec2 t)
{
    vec2 q=vec2(length(point.xz)-t.x,point.y);
    return vec4(color,length(q)-t.y);
}

vec3 op_i_trans(vec3 point,vec3 trans){
    return point-trans;
}

vec3 op_i_rot(vec3 point,vec3 rot){
    return point*euler_to_rotation(rot.x,rot.y,rot.z);
}
vec3 op_i_rot_ang(vec3 point,vec3 rot){
    return point*euler_to_rotation_ang(rot.x,rot.y,rot.z);
}

vec3 op_i_transrot(vec3 point,vec3 trans,vec3 rot){
    return(point-trans)*euler_to_rotation(rot.x,rot.y,rot.z);
}
vec3 op_i_transrot_ang(vec3 point,vec3 trans,vec3 rot){
    return(point-trans)*euler_to_rotation_ang(rot.x,rot.y,rot.z);
}

vec3 op_i_scale(vec3 point,float s){
    return point/s;
}

vec4 op_o_scale(vec4 sd,float s){
    return vec4(sd.xyz,sd.w*abs(s));
}

vec3 op_i_elongate(vec3 point,vec3 h){
    return point-clamp(point,-h,h);
}

vec4 op_o_round(vec4 sd,float rad){
    return vec4(sd.xyz,sd.w-rad);
}