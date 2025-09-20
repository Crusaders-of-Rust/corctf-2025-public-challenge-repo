#include <iostream>
#include <fstream>
#include <vector>
#include <string>
#include <unordered_map>
#include <cstdlib>
#include <ctime>

const std::string W = "./rockyou.txt";

bool load_file(std::vector<std::string>& a, std::unordered_map<std::string, size_t>& b) {
    std::ifstream f(W);
    if (!f.is_open()) {
        std::cerr << "Failed to open ./rockyou.txt\n";
        return false;
    }

    std::string x;
    size_t i = 0;
    while (std::getline(f, x)) {
        a.push_back(x);
        b[x] = i++;
    }

    f.close();
    return true;
}

std::vector<std::string> encode(const std::string& s, const std::vector<std::string>& wl, uint8_t k) {
    std::vector<std::string> r;

    r.push_back(wl[k]);

    for (size_t i = 0; i < s.size(); ++i) {
        uint8_t val = static_cast<uint8_t>(s[i]) ^ k;

        if (val >= wl.size()) {
            std::exit(1);
        }

        r.push_back(wl[val]);

        k = k ^ val ^ static_cast<uint8_t>(i);
    }

    return r;
}

uint8_t generate_key(time_t *timer) {
    std::srand(std::time(timer));
    return std::rand() % 256;
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " \"your message\"\n";
        return 1;
    }

    std::string in = argv[1];
    std::vector<std::string> wl;
    std::unordered_map<std::string, size_t> map;

    if (!load_file(wl, map)) {
        return 1;
    }

    if (wl.size() < 256) {
        std::cerr << "Wordlist too small.\n";
        return 1;
    }

    uint8_t key = generate_key(nullptr);

    auto result = encode(in, wl, key);

    std::cout << "Encoded output:\n";
    for (const auto& w : result) {
        std::cout << w << " ";
    }
    std::cout << "\n";

    return 0;
}
