public class Unscrambler {
    private static long HASH_MULTIPLIER = 13;
    private static long HASH_SEED = 0xDEADBEEFL;
    
    private static long hash_value;
    
    private static void init_hash() {
        hash_value = HASH_SEED;
    }
    
    private static void update_hash_inner(int num_iter, long data) {
        if (num_iter == 0) return;
        hash_value = hash_value * HASH_MULTIPLIER + data;
        update_hash_inner(num_iter -  1, data);
    }
    
    private static void update_hash(long data) {
        update_hash_inner(5, data);
    }
    
    private static long get_hash() {
        return hash_value;
    }
    
    private static char get_scramble(char input) {
        long hash_val = get_hash();
        boolean flip_sign = (((hash_val >>> 32) % 2) + 4) % 2 != 0;
        update_hash(0xFFL & (long) input);
        if (flip_sign) return (char) (0x20 ^ input);
        else return input;
    }
    
    private static char require_scramble(char output) {
        long hash_val = get_hash();
        boolean will_flip_sign = (((hash_val >>> 32) % 2) + 4) % 2 != 0;
        char input = will_flip_sign ? (char) (0x20 ^ output) : output;
        update_hash(0xFFL & (long) input);
        return input;
    }
    
    public static void main(String args[]) {
        System.out.println("initting");
        init_hash();
        
        String required_flag_body = "ahaHahhaaaaaaaahaHHahaaaaaaaahhaHaha";
        
        System.out.println("unscrambling " + required_flag_body);
        System.out.print("corctf{");
        for (char c : required_flag_body.toCharArray()) {
            System.out.print(require_scramble(c));
        }
        System.out.println("}");
        // corctf{aHAhaHHAaAaAAAAhAhhahAaAAAaAAhhaHahA}
        // echo "corctf{aHAhaHHAaAaAAAAhAhhahAaAAAaAAhhaHahA}" | ./bubble_vm program2.txt
    }
}