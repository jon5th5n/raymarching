#version 410
precision lowp float;

//---------------------------------------------------------------------------------------

vec4 com_union(vec4 one,vec4 two){
    return one.w<two.w?one:two;
}
vec4 com_smooth_union(vec4 one,vec4 two,float k){
    float h=clamp(.5+.5*(two.w-one.w)/k,0.,1.);
    float d=mix(two.w,one.w,h)-k*h*(1.-h);
    vec3 c=mix(two.xyz,one.xyz,h);
    return vec4(c,d);
}

vec4 com_intersect(vec4 one,vec4 two){
    return one.w>two.w?one:two;
}
vec4 com_smooth_intersect(vec4 one,vec4 two,float k){
    float h=clamp(.5-.5*(two.w-one.w)/k,0.,1.);
    float d=mix(two.w,one.w,h)+k*h*(1.-h);
    vec3 c=mix(two.xyz,one.xyz,h);
    return vec4(c,d);
}

vec4 com_substract(vec4 one,vec4 two){
    return-one.w>two.w?vec4(one.xyz,-one.w):two;
}
vec4 com_smooth_substract(vec4 one,vec4 two,float k){
    float h=clamp(.5-.5*(two.w+one.w)/k,0.,1.);
    float d=mix(two.w,-one.w,h)+k*h*(1.-h);
    vec3 c=mix(two.xyz,one.xyz,h);
    return vec4(c,d);
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

vec4 sdf_plane(vec3 point,vec3 color,vec3 n,float h)
{
    return vec4(color,dot(point,normalize(n))-h);
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
vec4 op_o_round_ang(vec4 sd,float ang){
    return vec4(sd.xyz,sd.w-ang*.017453292);
}

vec3 op_i_rep_inf(vec3 point,vec3 c){
    return mod(point+.5*c,c)-.5*c;
}

vec3 op_i_rep_lim(vec3 point,float c,vec3 l)
{
    return point-c*clamp(round(point/c),-l,l);
}

vec3 op_i_rep_polar(vec3 p,float repetitions){
    float angle=2*3.14159265/repetitions;
    float a=atan(p.y,p.x)+angle/2.;
    float r=length(p);
    a=mod(a,angle)-angle/2.;
    return vec3(cos(a)*r,sin(a)*r,p.z);
}