#version 410
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
#define Instructions Texture

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

// vec3 get_normal(vec3 p){
    //     float epsilon=.001;// arbitrary — should be smaller than any surface detail in your distance function, but not so small as to get lost in float precision
    //     float centerDistance=sdf_sphere(p,vec3(60.,0.,0.),30.);
    //     float xDistance=sdf_sphere(p+vec3(epsilon,0,0),vec3(60.,0.,0.),30.);
    //     float yDistance=sdf_sphere(p+vec3(0,epsilon,0),vec3(60.,0.,0.),30.);
    //     float zDistance=sdf_sphere(p+vec3(0,0,epsilon),vec3(60.,0.,0.),30.);
    //     vec3 normal=(vec3(xDistance,yDistance,zDistance)-centerDistance)/epsilon;
    
    //     return normal;
// }

// vec3 get_normal(in vec3 p)
// {
    //     const vec3 small_step=vec3(.001,0.,0.);
    
    //     float gradient_x=sdf_sphere(p+small_step.xyy,vec3(60.,0.,0.),30.)-sdf_sphere(p-small_step.xyy,vec3(60.,0.,0.),30.);
    //     float gradient_y=sdf_sphere(p+small_step.yxy,vec3(60.,0.,0.),30.)-sdf_sphere(p-small_step.yxy,vec3(60.,0.,0.),30.);
    //     float gradient_z=sdf_sphere(p+small_step.yyx,vec3(60.,0.,0.),30.)-sdf_sphere(p-small_step.yyx,vec3(60.,0.,0.),30.);
    
    //     vec3 normal=vec3(gradient_x,gradient_y,gradient_z);
    
    //     return normalize(normal);
// }

// float get_shading(vec3 p){
    //     vec3 light_source=vec3(0.,0.,50.);
    
    //     vec3 light_dir=light_source-p;
    
    //     return dot(get_normal(p),light_dir);
// }

//=======================================================================================

#define VIRTUAL_STACK_SIZE 50

uint op_stack[VIRTUAL_STACK_SIZE];
uint op_stack_size=0;
void op_stack_push(uint val){
    op_stack[op_stack_size]=val;
    op_stack_size++;
}
uint op_stack_pop(){
    op_stack_size--;
    return op_stack[op_stack_size];
}

vec4 res_stack[VIRTUAL_STACK_SIZE];
uint res_stack_size=0;
void res_stack_push(vec4 val){
    res_stack[res_stack_size]=val;
    res_stack_size++;
}
vec4 res_stack_pop(){
    res_stack_size--;
    return res_stack[res_stack_size];
}

//---------------------------------------------------------------------------------------

#define SDF_UNION 255
vec4 sdf_union(vec4 one,vec4 two){
    if(one[3]<two[3])return one;
    return two;
    // return vec4(0.,0.,0.,min(one[3],two[3]));
}
#define SDF_INTERSECTION 254
vec4 sdf_intersection(vec4 one,vec4 two){
    if(one[3]>two[3])return one;
    return two;
    // return vec4(0.,0.,0.,max(one[3],two[3]));
}
#define SDF_SUBSTRACTION 253
vec4 sdf_substraction(vec4 one,vec4 two){
    if(-one[3]>two[3])return vec4(one.xyz,-one[3]);
    return two;
}

//----------------------

#define SDF_SPHERE 1
float sdf_sphere(vec3 point,vec3 pos,float r){
    return length(pos-point)-r;
}

#define SDF_BOX 2
float sdf_box(vec3 point,vec3 pos,vec3 s)
{
    vec3 q=abs(pos-point)-s;
    return length(max(q,0.))+min(max(q.x,max(q.y,q.z)),0.);
}

//---------------------------------------------------------------------------------------

uint read_instruction_u8(inout uint instr_p){
    uint u32_index=uint(float(instr_p)/4.);
    uint u8_index=instr_p%4;
    
    float f32=texelFetch(Instructions,ivec2(u32_index,0),0)[u8_index];
    uint u8=uint(f32*255.);
    
    instr_p++;
    
    return u8;
}

uint read_instruction_u32(inout uint instr_p){
    uvec4 u8s=uvec4(read_instruction_u8(instr_p),read_instruction_u8(instr_p),read_instruction_u8(instr_p),read_instruction_u8(instr_p));
    uint u32=((u8s[0]<<24)+(u8s[1]<<16)+(u8s[2]<<8)+(u8s[3]));
    
    return u32;
}

float read_instruction_f32(inout uint instr_p){
    return uintBitsToFloat(read_instruction_u32(instr_p));
}

//----------------------

vec4 switch_evaluate_sdf(inout uint instr_p,uint instruction,vec3 point){
    vec3 color=vec3(read_instruction_f32(instr_p),read_instruction_f32(instr_p),read_instruction_f32(instr_p));
    float dist;
    switch(instruction){
        case SDF_SPHERE:
        dist=sdf_sphere(point,vec3(read_instruction_f32(instr_p),read_instruction_f32(instr_p),read_instruction_f32(instr_p)),read_instruction_f32(instr_p));
        break;
        
        case SDF_BOX:
        dist=sdf_box(point,vec3(read_instruction_f32(instr_p),read_instruction_f32(instr_p),read_instruction_f32(instr_p)),vec3(read_instruction_f32(instr_p),read_instruction_f32(instr_p),read_instruction_f32(instr_p)));
        break;
    }
    
    return vec4(color,dist);
}

vec4 march_instruction_tree(inout uint instr_p,vec3 point){
    uint instruction=read_instruction_u8(instr_p);
    if(instruction==0)return vec4(-1.,-1.,-1.,0.);
    
    op_stack_size=0;
    res_stack_size=0;
    
    uint res_count=0;
    
    while(true){
        if(instruction>=253)op_stack_push(instruction);
        else{
            res_stack_push(switch_evaluate_sdf(instr_p,instruction,point));
            res_count++;
        }
        
        if(res_count>=2){
            while(res_stack_size>=2){
                uint op=op_stack_pop();
                vec4 res;
                switch(op){
                    case SDF_UNION:
                    res=sdf_union(res_stack_pop(),res_stack_pop());
                    break;
                    
                    case SDF_INTERSECTION:
                    res=sdf_intersection(res_stack_pop(),res_stack_pop());
                    break;
                    
                    case SDF_SUBSTRACTION:
                    res=sdf_substraction(res_stack_pop(),res_stack_pop());
                    break;
                }
                res_stack_push(res);
            }
        }
        
        if(op_stack_size<=0)break;
        
        instruction=read_instruction_u8(instr_p);
    }
    
    return res_stack_pop();
}

vec4 march_instructions(vec3 point){
    uint instruction_pointer=0;
    
    vec4 smallest_distance=march_instruction_tree(instruction_pointer,point);
    while(true){
        vec4 result=march_instruction_tree(instruction_pointer,point);
        if(result[0]<0.)break;
        if(result[3]<smallest_distance[3])smallest_distance=result;
    }
    
    return smallest_distance;
}

vec4 march(){
    vec2 nuv=(uv*2.)-1.;
    
    float fovh=cam_fov;
    float fovv=cam_fov*(float(cam_size.y)/float(cam_size.x));
    
    vec3 ray=rotate(rotate(cam_direction,cam_up,(fovh/2.)*-nuv.x),cam_right,(fovv/2.)*-nuv.y);
    
    //-----
    
    vec4 color=vec4(.1804,.1804,.1804,1.);
    // for(float i=0.;i<cam_depth;i){
        //     vec3 point=cam_position+(ray*i);
        //     vec4 dist=march_instructions(point);
        //     if(dist[3]<=0.){
            //         color=vec4(dist.xyz,1.);
            //         // color=vec4(0.,1.,0.,1.);
            //         break;
        //     }
        //     i+=dist[3];
    // }
    
    float i=0.;
    while(true){
        vec3 point=cam_position+(ray*i);
        vec4 dist=march_instructions(point);
        if(dist[3]<=.1){
            color=vec4(dist.xyz,1.);
            // color=vec4(0.,1.,0.,1.);
            break;
        }
        
        i+=dist[3];
        if(i>=cam_depth)break;
    }
    
    return color;
}

// vec4 get_color(){
    //     vec2 nuv=(uv*2.)-1.;
    
    //     float fovh=cam_fov;
    //     float fovv=cam_fov*(float(cam_size.y)/float(cam_size.x));
    
    //     // float magw=tan(fovh/2.);
    //     // float magh=tan(fovv/2.);
    
    //     // vec3 ray=normalize(cam_direction+(cam_right*magw*nuv.x)+(cam_up*magh*-nuv.y));
    
    //     // vec3 ray=rotate(rotate(cam_direction,cam_up,(fovh/2.)*nuv.x),cam_right,(fovv/2.)*-nuv.y);
    //     vec3 ray=rotate(rotate(cam_direction,cam_up,(fovh/2.)*-nuv.x),cam_right,(fovv/2.)*-nuv.y);
    
    //     // vec3 ray=vec3(1.,nuv);
    
    //     //=====
    
    //     vec4 color=vec4(.1804,.1804,.1804,1.);
    //     for(uint i=0;i<cam_depth;i++){
        //         vec3 pos=cam_position+(ray*i);
        //         float d=sdf_sphere(pos,vec3(60.,0.,0.),30.);
        //         if(d<=0.){
            //             color=vec4(vec3(1.,0.,0.),1.);
            //             // color=vec4(abs(get_normal(pos)),1.);
            //             break;
        //         }
    //     }
    
    //     //=====
    
    //     return color;
// }

//=======================================================================================

void main(){
    // gl_FragColor=texelFetch(Instructions,ivec2(0,0),0);
    // read_data_u32(0);
    // gl_FragColor=vec4(uintBitsToFloat(read_data_u32(0)),0.,0.,1);
    
    // gl_FragColor=get_color();
    gl_FragColor=march();
    
    // vec3 pos=vec3(0.,0.,0.);
    // float r=15.;
    
    // uint instr_p=0;
    
    // uint op=read_instruction_u8(instr_p);
    // vec3 color=vec3(read_instruction_f32(instr_p),read_instruction_f32(instr_p),read_instruction_f32(instr_p));
    // vec3 pos=vec3(read_instruction_f32(instr_p),read_instruction_f32(instr_p),read_instruction_f32(instr_p));
    // float r=read_instruction_f32(instr_p);
    
    // float d=sdf_sphere(vec3(0.,((uv*2.)-1.)*25),pos,r);
    // vec4 d=march_instructions(vec3(0.,((uv*2.)-1.)*25));
    // gl_FragColor=vec4(d[3],0.,0.,1.);
    // uint i=0;
    // gl_FragColor=vec4(cam_depth/1000.,0.,0.,1.);
    // gl_FragColor=vec4(color,1.);
    
}